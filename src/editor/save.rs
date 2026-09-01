use super::BMM_BANNER;
use super::session::{EditorError, EditorOutcome, edit_text};
use crate::domain::PotentialBookmark;
use serde::Deserialize;

#[derive(Debug)]
pub(crate) enum SaveBookmarkEditorOutcome {
    Unchanged,
    Submitted(PotentialBookmark),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SaveBookmarkEditorError {
    #[error(transparent)]
    Editor(#[from] EditorError),
    #[error("couldn't parse editor document: {0}")]
    Parse(#[from] toml::de::Error),
}

pub(crate) fn get_save_bookmark_input(
    uri: &str,
    initial_title: Option<&str>,
    initial_tags: Option<&str>,
) -> Result<SaveBookmarkEditorOutcome, SaveBookmarkEditorError> {
    let document = SaveBookmarkDocument::new(initial_title, initial_tags);
    let initial_contents = render_save_bookmark_document(uri, &document);

    match edit_text(&initial_contents)? {
        EditorOutcome::Unchanged => Ok(SaveBookmarkEditorOutcome::Unchanged),
        EditorOutcome::Changed(edited_contents) => {
            let document = parse_save_bookmark_document(&edited_contents)?;
            Ok(SaveBookmarkEditorOutcome::Submitted(
                document.into_potential_bookmark(uri),
            ))
        }
    }
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
