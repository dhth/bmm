use crate::common::{HTML, IMPORT_FILE_FORMATS, IMPORT_UPPER_LIMIT, JSON, TXT};
use crate::domain::{
    DraftBookmark, DraftBookmarkError, DraftBookmarkErrors, PotentialImportedBookmark,
};
use crate::persistence::{DBError, SaveBookmarkOptions, create_or_update_bookmarks};
use select::document::Document;
use select::predicate::Name;
use sqlx::{Pool, Sqlite};
use std::io::Error as IOError;
use std::io::{BufRead, BufReader, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs::File, path::PathBuf};

type ParseResult = Result<Vec<DraftBookmark>, Vec<(usize, DraftBookmarkError)>>;

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error("file has no extension")]
    FileHasNoExtension,
    #[error("file doesn't exist")]
    FileDoesntExist,
    #[error("couldn't open file: {0}")]
    CouldntOpenFile(#[source] IOError),
    #[error("couldn't read file: {0}")]
    CouldntReadFile(#[source] IOError),
    #[error("couldn't parse HTML input: {0}")]
    CouldntParseHTMLInput(#[source] IOError),
    #[error("couldn't parse JSON input: {0}")]
    CouldntDeserializeJSONInput(#[from] serde_json::Error),
    #[error("file has too many bookmarks: {0} (maximum allowed at a time: {IMPORT_UPPER_LIMIT})")]
    TooManyBookmarks(usize),
    #[error("file format \"{0}\" not supported (supported formats: {IMPORT_FILE_FORMATS:?})")]
    FileFormatNotSupported(String),
    #[error("{}\n\n{}", errors.msg(), errors)]
    ValidationError { errors: DraftBookmarkErrors },
    #[error("couldn't save bookmarks to bmm's database: {0}")]
    SaveError(#[from] DBError),
    #[error("something unexpected happened: {0}")]
    UnexpectedError(String),
}

#[derive(Debug)]
pub struct ImportStats {
    pub num_bookmarks_imported: usize,
}

pub async fn import_bookmarks(
    pool: &Pool<Sqlite>,
    path: &str,
    reset_missing: bool,
    dry_run: bool,
    ignore_attribute_errors: bool,
) -> Result<Option<ImportStats>, ImportError> {
    let pathbuf = PathBuf::from(path);
    if !pathbuf.exists() {
        return Err(ImportError::FileDoesntExist);
    }

    let extension = pathbuf.extension().ok_or(ImportError::FileHasNoExtension)?;
    let extension_str = extension.to_str().ok_or(ImportError::UnexpectedError(
        "couldn't convert file extension to a string slice".into(),
    ))?;

    let parse_result = match extension_str {
        HTML => {
            let mut file = File::open(path).map_err(ImportError::CouldntOpenFile)?;
            let mut html_bytes = Vec::new();
            file.read_to_end(&mut html_bytes)
                .map_err(ImportError::CouldntReadFile)?;

            parse_html_content(html_bytes.as_slice(), ignore_attribute_errors)
                .map_err(ImportError::CouldntParseHTMLInput)?
        }
        TXT => {
            let file = File::open(path).map_err(ImportError::CouldntOpenFile)?;
            let reader = BufReader::new(file);
            let lines = reader
                .lines()
                .collect::<Result<Vec<String>, _>>()
                .map_err(ImportError::CouldntReadFile)?;

            parse_text_content(lines.as_slice())
        }
        JSON => {
            let mut file = File::open(path).map_err(ImportError::CouldntOpenFile)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .map_err(ImportError::CouldntReadFile)?;

            parse_json_content(bytes.as_slice(), ignore_attribute_errors)?
        }
        ext => {
            return Err(ImportError::FileFormatNotSupported(ext.into()));
        }
    };

    let draft_bookmarks = match parse_result {
        ParseResult::Ok(b) => b,
        ParseResult::Err(errs) => {
            return Err(ImportError::ValidationError {
                errors: DraftBookmarkErrors { errors: errs },
            });
        }
    };

    if draft_bookmarks.len() > IMPORT_UPPER_LIMIT {
        return Err(ImportError::TooManyBookmarks(draft_bookmarks.len()));
    }

    if dry_run {
        let output = serde_json::to_string_pretty(&draft_bookmarks).map_err(|e| {
            ImportError::UnexpectedError(format!(
                "couldn't serialize list of bookmarks to JSON: {e}"
            ))
        })?;
        println!("{output}");

        return Ok(None);
    }

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ImportError::UnexpectedError(format!("system time error: {e}")))?;
    let now = since_the_epoch.as_secs() as i64;
    let save_options = SaveBookmarkOptions {
        reset_missing_attributes: reset_missing,
        reset_tags: reset_missing,
    };
    create_or_update_bookmarks(pool, &draft_bookmarks, now, save_options).await?;

    Ok(Some(ImportStats {
        num_bookmarks_imported: draft_bookmarks.len(),
    }))
}

fn parse_html_content(
    bytes: &[u8],
    ignore_attribute_errors: bool,
) -> Result<ParseResult, std::io::Error> {
    let document = Document::from_read(bytes)?;
    let mut validation_errors = Vec::new();
    let mut draft_bookmarks = Vec::new();

    for (index, node) in document.find(Name("a")).enumerate() {
        let uri = node.attr("href").unwrap_or("");
        let title = node.text();
        let tags = node.attr("tags").unwrap_or("");
        let potential_bookmark =
            PotentialImportedBookmark::from((uri, Some(title.as_str()), Some(tags)));
        match DraftBookmark::try_from((potential_bookmark, ignore_attribute_errors)) {
            Ok(db) => {
                draft_bookmarks.push(db);
            }
            Err(e) => {
                validation_errors.push((index, e));
            }
        }
    }

    let result = if validation_errors.is_empty() {
        ParseResult::Ok(draft_bookmarks)
    } else {
        ParseResult::Err(validation_errors)
    };

    Ok(result)
}

fn parse_text_content(lines: &[String]) -> ParseResult {
    let mut validation_errors = Vec::new();
    let mut draft_bookmarks = Vec::new();
    for (index, uri) in lines.iter().enumerate() {
        let potential_bookmark = PotentialImportedBookmark {
            uri: uri.clone(),
            title: None,
            tags: None,
        };

        let db_result = DraftBookmark::try_from(potential_bookmark);
        match db_result {
            Ok(db) => draft_bookmarks.push(db),
            Err(e) => validation_errors.push((index, e)),
        }
    }

    if validation_errors.is_empty() {
        ParseResult::Ok(draft_bookmarks)
    } else {
        ParseResult::Err(validation_errors)
    }
}

fn parse_json_content(
    bytes: &[u8],
    ignore_attribute_errors: bool,
) -> Result<ParseResult, serde_json::Error> {
    let potential_bookmarks: Vec<PotentialImportedBookmark> = serde_json::from_slice(bytes)?;

    let mut validation_errors = Vec::new();
    let mut draft_bookmarks = Vec::new();
    for (index, pb) in potential_bookmarks.into_iter().enumerate() {
        let db_result = DraftBookmark::try_from((pb, ignore_attribute_errors));
        match db_result {
            Ok(db) => draft_bookmarks.push(db),
            Err(e) => validation_errors.push((index, e)),
        }
    }

    let result = if validation_errors.is_empty() {
        ParseResult::Ok(draft_bookmarks)
    } else {
        ParseResult::Err(validation_errors)
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_valid_html_content_works() {
        // GIVEN
        let content = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<!-- This is an automatically generated file.
It will be read and overwritten.
Do Not Edit! -->
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
<DT><A HREF="https://github.com/junegunn/fzf" ADD_DATE="1739262074" PRIVATE="0" TAGS="search,cli">junegunn/fzf: :cherry_blossom: A command-line fuzzy finder</A>
<DT><A HREF="https://github.com/serde-rs/serde" ADD_DATE="1739264074" LAST_MODIFIED="1739264074" TAGS="rust,github">serde-rs/serde: Serialization framework for Rust</A>
<DD>This article describes JavaScript for Automation, a new feature in OS X Yosemite.
</DL><p>
"#;

        // WHEN
        let result =
            parse_html_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN

        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags(), vec!["cli", "search"]);
        assert_eq!(
            draft_bookmarks[0].title(),
            Some("junegunn/fzf: :cherry_blossom: A command-line fuzzy finder")
        );
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags(), vec!["github", "rust"]);
        assert_eq!(
            draft_bookmarks[1].title(),
            Some("serde-rs/serde: Serialization framework for Rust")
        );
    }

    #[test]
    fn parsing_html_content_with_nested_folders_works() {
        // GIVEN
        let content = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
    <DT><H3>Folder 1</H3>
    <DL><p>
        <DT><H3>Subfolder 1</H3>
        <DL><p>
            <DT><A HREF="https://github.com/junegunn/fzf" TAGS="search,cli">junegunn/fzf: :cherry_blossom: A command-line fuzzy finder</A>
        </DL><p>
    </DL><p>
    <DT><H3>Folder 2</H3>
    <DL><p>
        <DT><A HREF="https://github.com/serde-rs/serde" TAGS="rust,github">serde-rs/serde: Serialization framework for Rust</A>
    </DL><p>
</DL><p>
"#;

        // WHEN
        let result =
            parse_html_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN
        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags(), vec!["cli", "search"]);
        assert_eq!(
            draft_bookmarks[0].title(),
            Some("junegunn/fzf: :cherry_blossom: A command-line fuzzy finder")
        );
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags(), vec!["github", "rust"]);
        assert_eq!(
            draft_bookmarks[1].title(),
            Some("serde-rs/serde: Serialization framework for Rust")
        );
    }

    #[test]
    fn parsing_html_content_with_missing_attributes_works() {
        // GIVEN
        let content = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<!-- This is an automatically generated file.
It will be read and overwritten.
Do Not Edit! -->
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
<DT><A HREF="https://github.com/junegunn/fzf" ADD_DATE="1739262074" PRIVATE="0">junegunn/fzf: :cherry_blossom: A command-line fuzzy finder</A>
<DT><A HREF="https://github.com/serde-rs/serde" ADD_DATE="1739264074" LAST_MODIFIED="1739264074" TAGS="rust,github"></A>
<DD>This article describes JavaScript for Automation, a new feature in OS X Yosemite.
</DL><p>
"#;

        // WHEN
        let result =
            parse_html_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN
        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags().len(), 0);
        assert_eq!(
            draft_bookmarks[0].title(),
            Some("junegunn/fzf: :cherry_blossom: A command-line fuzzy finder")
        );
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags(), vec!["github", "rust"]);
        assert!(draft_bookmarks[1].title().is_none(),);
    }

    #[test]
    fn parsing_valid_text_content_works() {
        // GIVEN
        let content = vec![
            "https://github.com/junegunn/fzf".to_string(),
            "https://github.com/serde-rs/serde".to_string(),
        ];

        // WHEN
        let result = parse_text_content(content.as_slice());
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN
        assert_eq!(draft_bookmarks.len(), 2);

        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags().len(), 0);
        assert!(draft_bookmarks[0].title().is_none(),);

        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags().len(), 0);
        assert!(draft_bookmarks[1].title().is_none(),);
    }

    #[test]
    fn parsing_valid_json_content_works() {
        // GIVEN
        let content = r#"
[
  {
    "tags": "cli,search",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder",
    "uri": "https://github.com/junegunn/fzf"
  },
  {
    "tags": "rust,github",
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result =
            parse_json_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN

        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags(), vec!["cli", "search"]);
        assert_eq!(
            draft_bookmarks[0].title(),
            Some("junegunn/fzf: :cherry_blossom: A command-line fuzzy finder")
        );
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags(), vec!["github", "rust"]);
        assert_eq!(
            draft_bookmarks[1].title(),
            Some("serde-rs/serde: Serialization framework for Rust")
        );
    }

    #[test]
    fn parsing_json_content_with_padded_tags_works() {
        // GIVEN
        let content = r#"
[
  {
    "tags": "cli, search",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder",
    "uri": "https://github.com/junegunn/fzf"
  },
  {
    "tags": "rust,   github   ",
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result =
            parse_json_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN

        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags(), vec!["cli", "search"]);
        assert_eq!(
            draft_bookmarks[0].title(),
            Some("junegunn/fzf: :cherry_blossom: A command-line fuzzy finder")
        );
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags(), vec!["github", "rust"]);
        assert_eq!(
            draft_bookmarks[1].title(),
            Some("serde-rs/serde: Serialization framework for Rust")
        );
    }

    #[test]
    fn parsing_json_content_with_missing_title_works() {
        // GIVEN
        let content = r#"
[
  {
    "tags": "cli,search",
    "title": "",
    "uri": "https://github.com/junegunn/fzf"
  },
  {
    "tags": "rust,github",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result =
            parse_json_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN

        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags(), vec!["cli", "search"]);
        assert!(draft_bookmarks[0].title().is_none());
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags(), vec!["github", "rust"]);
        assert!(draft_bookmarks[1].title().is_none());
    }

    #[test]
    fn parsing_json_content_with_missing_tags_works() {
        // GIVEN
        let content = r#"
[
  {
    "tags": "",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder",
    "uri": "https://github.com/junegunn/fzf"
  },
  {
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result =
            parse_json_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let draft_bookmarks = result.expect("should've returned draft bookmarks");

        // THEN

        assert_eq!(draft_bookmarks.len(), 2);
        assert_eq!(draft_bookmarks[0].uri(), "https://github.com/junegunn/fzf");
        assert_eq!(draft_bookmarks[0].tags().len(), 0);
        assert_eq!(
            draft_bookmarks[0].title(),
            Some("junegunn/fzf: :cherry_blossom: A command-line fuzzy finder")
        );
        assert_eq!(
            draft_bookmarks[1].uri(),
            "https://github.com/serde-rs/serde"
        );
        assert_eq!(draft_bookmarks[1].tags().len(), 0);
        assert_eq!(
            draft_bookmarks[1].title(),
            Some("serde-rs/serde: Serialization framework for Rust")
        );
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn parsing_html_content_with_incorrect_uris_returns_validation_errors() {
        // GIVEN
        let content = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<!-- This is an automatically generated file.
It will be read and overwritten.
Do Not Edit! -->
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
<DT><A HREF="" ADD_DATE="1739262074" PRIVATE="0" TAGS="search,cli">junegunn/fzf: :cherry_blossom: A command-line fuzzy finder</A>
<DT><A HREF="https://github.com/serde-rs/serde" ADD_DATE="1739264074" LAST_MODIFIED="1739264074" TAGS="rust,github">serde-rs/serde: Serialization framework for Rust</A>
<DD>This article describes JavaScript for Automation, a new feature in OS X Yosemite.
</DL><p>
"#;

        // WHEN
        let result =
            parse_html_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let validation_errors = result.expect_err("should've returned validation errors");

        // THEN
        assert_eq!(validation_errors.len(), 1);
    }

    #[test]
    fn parsing_html_content_with_incorrect_tags_returns_validation_errors() {
        // GIVEN
        let content = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<!-- This is an automatically generated file.
It will be read and overwritten.
Do Not Edit! -->
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
<DT><A HREF="https://github.com/junegunn/fzf" ADD_DATE="1739262074" PRIVATE="0" TAGS="invalid!!!tag,cli">junegunn/fzf: :cherry_blossom: A command-line fuzzy finder</A>
<DT><A HREF="https://github.com/serde-rs/serde" ADD_DATE="1739264074" LAST_MODIFIED="1739264074" TAGS="rust,github">serde-rs/serde: Serialization framework for Rust</A>
<DD>This article describes JavaScript for Automation, a new feature in OS X Yosemite.
</DL><p>
"#;

        // WHEN
        let result =
            parse_html_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let validation_errors = result.expect_err("should've returned validation errors");

        // THEN
        assert_eq!(validation_errors.len(), 1);
    }

    #[test]
    fn parsing_text_content_returns_validation_errors_for_incorrect_uris() {
        // GIVEN
        let content = vec![
            "https/github.com/junegunn/fzf".to_string(),
            "https://github.com/serde-rs/serde".to_string(),
        ];

        // WHEN
        let validation_errors = parse_text_content(content.as_slice())
            .expect_err("should've returned validation errors");

        // THEN
        assert_eq!(validation_errors.len(), 1);
    }

    #[test]
    fn parsing_invalid_json_fails() {
        // GIVEN
        let content = r#"
[
  {
    "tags": "",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder",
    "uri": "https://github.com/junegunn/fzf"
  }
  {
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result = parse_json_content(content.as_bytes(), false);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn parsing_json_content_without_mandatory_fields_fails() {
        // GIVEN
        let content = r#"
[
  {
    "tags": "",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder"
  }
  {
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result = parse_json_content(content.as_bytes(), false);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn parsing_json_content_with_empty_uri_fails() {
        // GIVEN
        let content = r#"
[
  {
    "uri": "",
    "tags": "",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder"
  },
  {
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result =
            parse_json_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let validation_errors = result.expect_err("should've returned validation errors");

        // THEN
        assert_eq!(validation_errors.len(), 1);
    }

    #[test]
    fn parsing_json_content_with_invalid_tags_fails() {
        // GIVEN
        let content = r#"
[
  {
    "uri": "https://github.com/junegunn/fzf",
    "tags": "invalid tag",
    "title": "junegunn/fzf: :cherry_blossom: A command-line fuzzy finder"
  },
  {
    "title": "serde-rs/serde: Serialization framework for Rust",
    "uri": "https://github.com/serde-rs/serde"
  }
]
"#;

        // WHEN
        let result =
            parse_json_content(content.as_bytes(), false).expect("parsing should've succeeded");
        let validation_errors = result.expect_err("should've returned validation errors");

        // THEN
        assert_eq!(validation_errors.len(), 1);
    }
}
