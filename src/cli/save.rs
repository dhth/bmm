use crate::common::{ENV_VAR_BMM_EDITOR, ENV_VAR_EDITOR};
use crate::domain::{DraftBookmark, DraftBookmarkError, PotentialBookmark};
use crate::persistence::{
    DBError, SaveBookmarkOptions, create_or_update_bookmark, get_bookmark_with_exact_uri,
};
use crate::service::{FetchUriDetailsError, fetch_uri_details};
use regex::{Error as RegexError, Regex};
use sqlx::{Pool, Sqlite};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;
use which::{Error as WhichError, which};

#[derive(thiserror::Error, Debug)]
pub enum SaveBookmarkError {
    #[error("couldn't check if uri already saved: {0}")]
    CouldntCheckIfBookmarkExists(DBError),
    #[error("uri already saved")]
    UriAlreadySaved,
    #[error(transparent)]
    CouldntUseTextEditor(#[from] CouldntGetDetailsViaEditorError),
    #[error(transparent)]
    BookmarkDetailsAreInvalid(#[from] DraftBookmarkError),
    #[error(transparent)]
    CouldntSaveBookmark(DBError),
    #[error("something unexpected happened: {0}")]
    UnexpectedError(String),
    #[error(transparent)]
    CouldntFetchDetails(#[from] FetchUriDetailsError),
    #[error("no details fetched from remote server")]
    NoDetailsFetched,
}

#[derive(thiserror::Error, Debug)]
pub enum CouldntGetDetailsViaEditorError {
    #[error("couldn't create temporary file (to be opened in text editor): {0}")]
    CreateTempFile(std::io::Error),
    #[error("couldn't open temporary file (to be opened in text editor): {0}")]
    OpenTempFile(std::io::Error),
    #[error("couldn't write contents to temporary file (to be opened in text editor): {0}")]
    WriteToTempFile(std::io::Error),
    #[error("couldn't find editor executable \"{0}\": {2}")]
    CouldntFindEditorExe(String, String, WhichError),
    #[error("couldn't open text editor ({0}): {1}")]
    OpenTextEditor(PathBuf, std::io::Error),
    #[error("couldn't read contents of temporary file: {0}")]
    ReadTempFileContents(std::io::Error),
    #[error("editor environment variable \"{0}\" is invalid")]
    InvalidEditorEnvVar(String),
    #[error("no editor configured")]
    NoEditorConfigured,
    #[error("couldn't parse text entered via editor: {0}")]
    ParsingEditorText(#[from] ParsingTempFileContentError),
}

#[derive(thiserror::Error, Debug)]
pub enum ParsingTempFileContentError {
    #[error("bmm's internal regex is incorrect: {0}")]
    IncorrectRegexError(#[from] RegexError),
    #[error("one or more input is missing")]
    InputMissing,
}

pub struct SaveConfig {
    pub use_editor: bool,
    pub fail_if_uri_already_saved: bool,
    pub reset_missing: bool,
    pub ignore_attribute_errors: bool,
    pub fetch: bool,
}

pub async fn save_bookmark(
    pool: &Pool<Sqlite>,
    uri: String,
    title: Option<String>,
    tags: &[String],
    save_config: SaveConfig,
) -> Result<(), SaveBookmarkError> {
    let maybe_existing_bookmark = get_bookmark_with_exact_uri(pool, &uri)
        .await
        .map_err(SaveBookmarkError::CouldntCheckIfBookmarkExists)?;

    let bookmark_exists = maybe_existing_bookmark.is_some();

    if save_config.fail_if_uri_already_saved && bookmark_exists {
        return Err(SaveBookmarkError::UriAlreadySaved);
    }

    if maybe_existing_bookmark.is_some()
        && !save_config.fetch
        && !save_config.use_editor
        && !save_config.reset_missing
        && title.is_none()
        && tags.is_empty()
    {
        println!("nothing to update!");
        return Ok(());
    }

    let title_to_use = match title {
        Some(t) => Some(t),
        None => {
            if save_config.fetch {
                fetch_uri_details(&uri).await?
            } else if save_config.reset_missing {
                None
            } else if save_config.use_editor {
                maybe_existing_bookmark
                    .as_ref()
                    .and_then(|b| b.title.clone())
            } else {
                None
            }
        }
    };

    if !save_config.use_editor && save_config.fetch && title_to_use.is_none() {
        return Err(SaveBookmarkError::NoDetailsFetched);
    }

    let tags_to_use = if !tags.is_empty() {
        Some(tags.join(",").to_string())
    } else if save_config.reset_missing {
        None
    } else if save_config.use_editor {
        maybe_existing_bookmark.and_then(|b| b.tags)
    } else {
        None
    };

    let draft_bookmark = match save_config.use_editor {
        true => match bookmark_exists {
            true => {
                let (title, tags) = get_bookmark_update_details_from_temp_file(
                    uri.as_str(),
                    title_to_use.as_deref(),
                    tags_to_use.as_deref(),
                )?;

                let potential_bookmark =
                    PotentialBookmark::from((uri.as_str(), title.as_deref(), tags.as_deref()));

                DraftBookmark::try_from((potential_bookmark, save_config.ignore_attribute_errors))?
            }
            false => {
                let potential_bookmark = get_new_bookmark_details_from_temp_file(
                    &uri,
                    title_to_use.as_deref(),
                    tags_to_use.as_deref(),
                )?;

                DraftBookmark::try_from((potential_bookmark, save_config.ignore_attribute_errors))?
            }
        },
        false => {
            let potential_bookmark =
                PotentialBookmark::from((uri.clone(), title_to_use, tags_to_use));
            DraftBookmark::try_from((potential_bookmark, save_config.ignore_attribute_errors))?
        }
    };

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SaveBookmarkError::UnexpectedError(format!("system time error: {}", e)))?;
    let now = since_the_epoch.as_secs() as i64;
    let save_options = SaveBookmarkOptions {
        reset_missing_attributes: save_config.reset_missing,
        reset_tags: save_config.reset_missing,
    };
    create_or_update_bookmark(pool, &draft_bookmark, now, save_options)
        .await
        .map_err(SaveBookmarkError::CouldntSaveBookmark)?;

    Ok(())
}

fn get_bookmark_update_details_from_temp_file(
    uri: &str,
    title: Option<&str>,
    tags: Option<&str>,
) -> Result<(Option<String>, Option<String>), CouldntGetDetailsViaEditorError> {
    let tmp_dir = tempdir().map_err(CouldntGetDetailsViaEditorError::CreateTempFile)?;

    let tmp_file_path = tmp_dir.path().join("bmm-edit.txt");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp_file_path)
        .map_err(CouldntGetDetailsViaEditorError::OpenTempFile)?;

    let file_contents = get_update_bookmark_tmp_file_contents(uri, title, tags);
    file.write_all(file_contents.as_bytes())
        .map_err(CouldntGetDetailsViaEditorError::WriteToTempFile)?;

    let (editor_exe, env_var_used) = get_text_editor_exe()?;

    let editor_exe_path = which(&editor_exe).map_err(|e| {
        CouldntGetDetailsViaEditorError::CouldntFindEditorExe(editor_exe, env_var_used, e)
    })?;

    let _ = Command::new(&editor_exe_path)
        .arg(&tmp_file_path)
        .status()
        .map_err(|e| CouldntGetDetailsViaEditorError::OpenTextEditor(editor_exe_path, e))?;

    let mut modified_file =
        File::open(&tmp_file_path).map_err(CouldntGetDetailsViaEditorError::OpenTempFile)?;
    let mut modified_contents = String::new();
    modified_file
        .read_to_string(&mut modified_contents)
        .map_err(CouldntGetDetailsViaEditorError::ReadTempFileContents)?;

    Ok(parse_bookmark_update_temp_file_content(&modified_contents)?)
}

fn get_new_bookmark_details_from_temp_file(
    uri: &str,
    title: Option<&str>,
    tags: Option<&str>,
) -> Result<PotentialBookmark, CouldntGetDetailsViaEditorError> {
    let tmp_dir = tempdir().map_err(CouldntGetDetailsViaEditorError::CreateTempFile)?;

    let tmp_file_path = tmp_dir.path().join("bmm-new.txt");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp_file_path)
        .map_err(CouldntGetDetailsViaEditorError::OpenTempFile)?;

    let file_contents = get_create_bookmark_tmp_file_contents(uri, title, tags);
    file.write_all(file_contents.as_bytes())
        .map_err(CouldntGetDetailsViaEditorError::WriteToTempFile)?;

    let (editor_exe, env_var_used) = get_text_editor_exe()?;

    let editor_exe_path = which(&editor_exe).map_err(|e| {
        CouldntGetDetailsViaEditorError::CouldntFindEditorExe(editor_exe, env_var_used, e)
    })?;

    let _ = Command::new(&editor_exe_path)
        .arg(&tmp_file_path)
        .status()
        .map_err(|e| CouldntGetDetailsViaEditorError::OpenTextEditor(editor_exe_path, e))?;

    let mut modified_file =
        File::open(&tmp_file_path).map_err(CouldntGetDetailsViaEditorError::OpenTempFile)?;
    let mut modified_contents = String::new();
    modified_file
        .read_to_string(&mut modified_contents)
        .map_err(CouldntGetDetailsViaEditorError::ReadTempFileContents)?;

    let (uri, title, tags) = parse_new_bookmark_temp_file_content(&modified_contents)?;
    Ok(PotentialBookmark::from((
        &uri,
        title.as_ref(),
        tags.as_ref(),
    )))
}

fn get_text_editor_exe() -> Result<(String, String), CouldntGetDetailsViaEditorError> {
    fn get_env_var(key: &str) -> Result<String, CouldntGetDetailsViaEditorError> {
        match std::env::var(key) {
            Ok(v) => Ok(v),
            Err(std::env::VarError::NotPresent) => Ok("".to_string()),
            Err(std::env::VarError::NotUnicode(_)) => Err(
                CouldntGetDetailsViaEditorError::InvalidEditorEnvVar(key.into()),
            ),
        }
    }

    let bmm_text_editor = get_env_var(ENV_VAR_BMM_EDITOR)?;
    if !bmm_text_editor.trim().is_empty() {
        return Ok((bmm_text_editor, ENV_VAR_BMM_EDITOR.to_string()));
    }

    let text_editor = get_env_var(ENV_VAR_EDITOR)?;
    if !text_editor.trim().is_empty() {
        return Ok((text_editor, ENV_VAR_EDITOR.to_string()));
    }

    Err(CouldntGetDetailsViaEditorError::NoEditorConfigured)
}

fn get_update_bookmark_tmp_file_contents(
    uri: &str,
    title: Option<&str>,
    tags: Option<&str>,
) -> String {
    format!(
        r#"
       __             
      / /  __ _  __ _ 
     / _ \/  ' \/  ' \
    /_.__/_/_/_/_/_/_/


# Saving this file will update the bookmark.
# You will enter information on lines between ">>>" and "<<<"

URI (read only):
{}

Title:
>>>
{}
<<<

Comma separate tags:
>>>
{}
<<<
"#,
        uri,
        title.unwrap_or_default(),
        tags.unwrap_or_default(),
    )
}

fn get_create_bookmark_tmp_file_contents(
    uri: &str,
    title: Option<&str>,
    tags: Option<&str>,
) -> String {
    format!(
        r#"
       __             
      / /  __ _  __ _ 
     / _ \/  ' \/  ' \
    /_.__/_/_/_/_/_/_/


# Saving this file will create a new bookmark.
# You will enter information on lines between ">>>" and "<<<"

URI: 
>>>
{}
<<<

Title: 
>>>
{}
<<<

Comma separated tags:
>>>
{}
<<<
"#,
        uri,
        title.unwrap_or_default(),
        tags.unwrap_or_default(),
    )
}

fn parse_bookmark_update_temp_file_content(
    input: &str,
) -> Result<(Option<String>, Option<String>), ParsingTempFileContentError> {
    let re = Regex::new(r">>>\s*\n(.*?)\n\s*<<<")?;
    let captures: Vec<_> = re.captures_iter(input).collect();

    if captures.len() < 2 {
        return Err(ParsingTempFileContentError::InputMissing);
    }

    let title_line = captures[0][1].trim();
    let title = match title_line.is_empty() {
        true => None,
        false => Some(title_line.to_string()),
    };

    let tags_line = captures[1][1].trim();
    let tags = match tags_line.is_empty() {
        true => None,
        false => Some(tags_line.to_string()),
    };

    Ok((title, tags))
}

fn parse_new_bookmark_temp_file_content(
    input: &str,
) -> Result<(String, Option<String>, Option<String>), ParsingTempFileContentError> {
    let re = Regex::new(r">>>\s*\n(.*?)\n\s*<<<")?;
    let captures: Vec<_> = re.captures_iter(input).collect();

    if captures.len() < 3 {
        return Err(ParsingTempFileContentError::InputMissing);
    }

    let uri = captures[0][1].trim().to_string();

    let title_line = captures[1][1].trim();
    let title = match title_line.is_empty() {
        true => None,
        false => Some(title_line.to_string()),
    };

    let tags_line = captures[2][1].trim();
    let tags = match tags_line.is_empty() {
        true => None,
        false => Some(tags_line.to_string()),
    };

    Ok((uri, title, tags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_temp_file_content_for_bookmark_update_works() {
        // GIVEN
        let temp_file_content = r#"
       __             
      / /  __ _  __ _ 
     / _ \/  ' \/  ' \
    /_.__/_/_/_/_/_/_/


# Saving this file will update the bookmark.
# You will enter information on lines between ">>>" and "<<<"

URI (read only):
https://someuri.com

Title:
>>>
Uri title goes here
<<<

Comma separate tags:
>>>
tag1,tag2,tag3
<<<
"#;

        // WHEN
        let (title, tags) = parse_bookmark_update_temp_file_content(temp_file_content)
            .expect("parsing should've succeeded");

        // THEN
        assert_eq!(title.as_deref(), Some("Uri title goes here"));
        assert_eq!(tags.as_deref(), Some("tag1,tag2,tag3"));
    }

    #[test]
    fn parsing_update_bookmark_temp_file_content_with_empty_title_works() {
        // GIVEN
        let temp_file_content = r#"
URI (read only):
https://someuri.com

Title:
>>>
       
<<<

Comma separate tags:
>>>
tag1,tag2,tag3
<<<
"#;

        // WHEN
        let (title, _) = parse_bookmark_update_temp_file_content(temp_file_content)
            .expect("parsing should've succeeded");

        // THEN
        assert!(title.is_none());
    }

    #[test]
    fn parsing_update_bookmark_temp_file_content_with_empty_tags_line_works() {
        // GIVEN
        let temp_file_content = r#"
URI (read only):
https://someuri.com

Title:
>>>
Uri title goes here
<<<

Comma separate tags:
>>>
     
<<<
"#;

        // WHEN
        let (_, tags) = parse_bookmark_update_temp_file_content(temp_file_content)
            .expect("parsing should've succeeded");

        // THEN
        assert!(tags.is_none());
    }

    #[test]
    fn parsing_temp_file_content_for_new_bookmark_works() {
        // GIVEN
        let temp_file_content = r#"
       __             
      / /  __ _  __ _ 
     / _ \/  ' \/  ' \
    /_.__/_/_/_/_/_/_/


# Saving this file will create a new bookmark.
# You will enter information on lines between ">>>" and "<<<"

URI: 
>>>
https://someuri.com
<<<

Title: 
>>>
Title goes here
<<<

Comma separated tags:
>>>
tag1,tag2,tag3
<<<
"#;

        // WHEN
        let (uri, title, tags) = parse_new_bookmark_temp_file_content(temp_file_content)
            .expect("parsing should've succeeded");

        // THEN
        assert_eq!(uri.as_str(), "https://someuri.com");
        assert_eq!(title.as_deref(), Some("Title goes here"));
        assert_eq!(tags.as_deref(), Some("tag1,tag2,tag3"));
    }

    #[test]
    fn parsing_temp_file_content_for_new_bookmark_with_empty_title_works() {
        // GIVEN
        let temp_file_content = r#"
       __             
      / /  __ _  __ _ 
     / _ \/  ' \/  ' \
    /_.__/_/_/_/_/_/_/


# Saving this file will create a new bookmark.
# You will enter information on lines between ">>>" and "<<<"

URI: 
>>>
https://someuri.com
<<<

Title: 
>>>

<<<

Comma separated tags:
>>>
tag1,tag2,tag3
<<<
"#;

        // WHEN
        let (uri, title, tags) = parse_new_bookmark_temp_file_content(temp_file_content)
            .expect("parsing should've succeeded");

        // THEN
        assert_eq!(uri.as_str(), "https://someuri.com");
        assert!(title.is_none());
        assert_eq!(tags.as_deref(), Some("tag1,tag2,tag3"));
    }

    #[test]
    fn parsing_temp_file_content_for_new_bookmark_with_empty_tags_works() {
        // GIVEN
        let temp_file_content = r#"
       __             
      / /  __ _  __ _ 
     / _ \/  ' \/  ' \
    /_.__/_/_/_/_/_/_/


# Saving this file will create a new bookmark.
# You will enter information on lines between ">>>" and "<<<"

URI: 
>>>
https://someuri.com
<<<

Title: 
>>>
Title goes here
<<<

Comma separated tags:
>>>

<<<
"#;

        // WHEN
        let (uri, title, tags) = parse_new_bookmark_temp_file_content(temp_file_content)
            .expect("parsing should've succeeded");

        // THEN
        assert_eq!(uri.as_str(), "https://someuri.com");
        assert_eq!(title.as_deref(), Some("Title goes here"));
        assert!(tags.is_none());
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn parsing_update_bookmark_temp_file_without_title_line_fails() {
        // GIVEN
        let temp_file_content = r#"
URI (read only):
https://someuri.com

Title:
>>>
<<<

Comma separate tags:
>>>
     
<<<
"#;

        // WHEN
        let result = parse_bookmark_update_temp_file_content(temp_file_content);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn parsing_update_bookmark_temp_file_without_tags_line_fails() {
        // GIVEN
        let temp_file_content = r#"
URI (read only):
https://someuri.com

Title:
>>>
Title goes here
<<<

Comma separate tags:
>>>
<<<
"#;

        // WHEN
        let result = parse_bookmark_update_temp_file_content(temp_file_content);

        // THEN
        assert!(result.is_err());
    }
}
