use crate::domain::PotentialBookmark;
use serde::Deserialize;

const BMM_BANNER: &str = r#"#        __
#       / /  __ _  __ _
#      / _ \/  ' \/  ' \
#     /_.__/_/_/_/_/_/_/
#"#;

#[derive(Debug, Deserialize)]
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
}
