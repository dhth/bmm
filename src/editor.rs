use crate::common::{ENV_VAR_BMM_EDITOR, ENV_VAR_EDITOR};
use crate::domain::PotentialBookmark;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use tempfile::tempdir;
use which::{Error as WhichError, which};

const BMM_BANNER: &str = r#"#        __
#       / /  __ _  __ _
#      / _ \/  ' \/  ' \
#     /_.__/_/_/_/_/_/_/
#"#;

#[derive(Debug, PartialEq, Eq)]
enum EditorOutcome {
    Unchanged,
    Changed(String),
}

#[derive(Debug, thiserror::Error)]
enum EditorError {
    #[error("couldn't create temporary directory for editor file: {0}")]
    CreateTempDir(std::io::Error),
    #[error("couldn't write temporary editor file: {0}")]
    WriteTempFile(std::io::Error),
    #[error("editor environment variable \"{0}\" is invalid")]
    InvalidEditorEnvVar(String),
    #[error("no editor configured")]
    NoEditorConfigured,
    #[error("couldn't find editor executable \"{0}\": {2}")]
    CouldntFindEditorExe(String, String, WhichError),
    #[error("couldn't open text editor ({0}): {1}")]
    OpenTextEditor(PathBuf, std::io::Error),
    #[error("text editor exited unsuccessfully: {0}")]
    EditorFailed(ExitStatus),
    #[error("couldn't read temporary editor file: {0}")]
    ReadTempFile(std::io::Error),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SaveBookmarkDocument {
    title: String,
    tags: String,
}

impl SaveBookmarkDocument {
    fn new(title: Option<&str>, tags: Option<&str>) -> Self {
        Self {
            title: title.unwrap_or_default().to_string(),
            tags: tags.unwrap_or_default().to_string(),
        }
    }

    fn into_potential_bookmark(self, uri: &str) -> PotentialBookmark {
        PotentialBookmark::from((uri.to_string(), Some(self.title), Some(self.tags)))
    }
}

fn render_save_bookmark_document(uri: &str, document: &SaveBookmarkDocument) -> String {
    let uri = uri.replace('\n', "\n# ").replace('\r', "");
    let title = toml::Value::String(document.title.clone());
    let tags = toml::Value::String(document.tags.clone());

    format!(
        r#"{BMM_BANNER}
# This bookmark will be saved for:
# {uri}

# Set this to an empty string to remove the title.
title = {title}

# Separate tags with commas. Set this to an empty string to remove all tags.
tags = {tags}
"#
    )
}

fn parse_save_bookmark_document(input: &str) -> Result<SaveBookmarkDocument, toml::de::Error> {
    toml::from_str(input)
}

fn edit_text(initial_contents: &str) -> Result<EditorOutcome, EditorError> {
    let temp_dir = tempdir().map_err(EditorError::CreateTempDir)?;
    let file_path = temp_dir.path().join("bmm.toml");
    std::fs::write(&file_path, initial_contents).map_err(EditorError::WriteTempFile)?;

    let (editor_exe, env_var_used) = get_text_editor_exe()?;
    let editor_exe_path = which(&editor_exe)
        .map_err(|error| EditorError::CouldntFindEditorExe(editor_exe, env_var_used, error))?;

    let status = Command::new(&editor_exe_path)
        .arg(&file_path)
        .status()
        .map_err(|error| EditorError::OpenTextEditor(editor_exe_path, error))?;

    if !status.success() {
        return Err(EditorError::EditorFailed(status));
    }

    let edited_contents = std::fs::read_to_string(file_path).map_err(EditorError::ReadTempFile)?;
    if edited_contents == initial_contents {
        Ok(EditorOutcome::Unchanged)
    } else {
        Ok(EditorOutcome::Changed(edited_contents))
    }
}

fn get_text_editor_exe() -> Result<(String, String), EditorError> {
    fn get_env_var(key: &str) -> Result<String, EditorError> {
        match std::env::var(key) {
            Ok(value) => Ok(value),
            Err(std::env::VarError::NotPresent) => Ok(String::new()),
            Err(std::env::VarError::NotUnicode(_)) => {
                Err(EditorError::InvalidEditorEnvVar(key.to_string()))
            }
        }
    }

    let bmm_editor = get_env_var(ENV_VAR_BMM_EDITOR)?;
    if !bmm_editor.trim().is_empty() {
        return Ok((bmm_editor, ENV_VAR_BMM_EDITOR.to_string()));
    }

    let editor = get_env_var(ENV_VAR_EDITOR)?;
    if !editor.trim().is_empty() {
        return Ok((editor, ENV_VAR_EDITOR.to_string()));
    }

    Err(EditorError::NoEditorConfigured)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn rendering_save_bookmark_document_works() {
        let document = SaveBookmarkDocument::new(Some("The Rust guide"), Some("rust,reference"));

        let rendered = render_save_bookmark_document("https://example.com/rust", &document);

        assert_snapshot!(rendered);
    }

    #[test]
    fn rendering_save_bookmark_document_escapes_toml_strings() {
        let document = SaveBookmarkDocument::new(
            Some("A \"quoted\" title\nwith a second line\\path"),
            Some("rust,tab\tvalue"),
        );

        let rendered = render_save_bookmark_document("https://example.com/rust", &document);

        assert_snapshot!(rendered);
    }

    #[test]
    fn parsing_save_bookmark_document_works() {
        let input = r#"
# Fields can be reordered and surrounded by comments.
tags = "rust,reference"

# The title may contain TOML escapes.
title = "A \"quoted\" title\nwith a second line\\path"
"#;

        let document = parse_save_bookmark_document(input).expect("document should be valid TOML");

        assert_eq!(
            document,
            SaveBookmarkDocument {
                title: "A \"quoted\" title\nwith a second line\\path".to_string(),
                tags: "rust,reference".to_string(),
            }
        );
    }

    #[test]
    fn parsing_save_bookmark_document_with_empty_fields_works() {
        let document = parse_save_bookmark_document(
            r#"
title = ""
tags = ""
"#,
        )
        .expect("document should be valid TOML");

        assert_eq!(document, SaveBookmarkDocument::new(None, None));
    }

    #[test]
    fn parsing_save_bookmark_document_rejects_invalid_documents() {
        let invalid_documents = [
            ("empty document", ""),
            ("missing title", r#"tags = "rust""#),
            ("missing tags", r#"title = "Rust""#),
            (
                "unknown field",
                r#"
title = "Rust"
tags = "rust"
description = "A description"
"#,
            ),
            (
                "uri field",
                r#"
uri = "https://example.com/rust"
title = "Rust"
tags = "rust"
"#,
            ),
            (
                "non-string title",
                r#"
title = 42
tags = "rust"
"#,
            ),
            (
                "non-string tags",
                r#"
title = "Rust"
tags = ["rust"]
"#,
            ),
            (
                "duplicate field",
                r#"
title = "Rust"
title = "The Rust guide"
tags = "rust"
"#,
            ),
            (
                "malformed TOML",
                r#"
title = "Rust
tags = "rust"
"#,
            ),
        ];

        for (case, input) in invalid_documents {
            assert!(
                parse_save_bookmark_document(input).is_err(),
                "{case} should be rejected"
            );
        }
    }

    #[test]
    fn prefilled_save_bookmark_document_round_trips() {
        let original = SaveBookmarkDocument::new(
            Some("A \"quoted\" title\nwith a second line\\path"),
            Some("rust,tab\tvalue"),
        );

        let rendered = render_save_bookmark_document("https://example.com/rust", &original);
        let parsed = parse_save_bookmark_document(&rendered)
            .expect("rendered document should be valid TOML");

        assert_eq!(parsed, original);
    }

    #[test]
    fn empty_save_bookmark_document_round_trips() {
        let original = SaveBookmarkDocument::new(None, None);

        let rendered = render_save_bookmark_document("https://example.com/rust", &original);
        let parsed = parse_save_bookmark_document(&rendered)
            .expect("rendered document should be valid TOML");

        assert_eq!(parsed, original);
    }
}
