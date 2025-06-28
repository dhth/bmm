use super::tags::{TAG_REGEX_STR, Tag};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

const TAG_TITLE_MAX_LENGTH: usize = 500;

#[derive(Debug, Serialize)]
pub struct DraftBookmark {
    uri: String,
    title: Option<String>,
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct PotentialBookmark {
    pub uri: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PotentialImportedBookmark {
    pub uri: String,
    pub title: Option<String>,
    pub tags: Option<String>,
}

impl From<PotentialImportedBookmark> for PotentialBookmark {
    fn from(bookmark: PotentialImportedBookmark) -> Self {
        Self {
            uri: bookmark.uri,
            title: bookmark.title,
            tags: bookmark
                .tags
                .unwrap_or_default()
                .split(",")
                .map(|t| t.to_string())
                .collect::<Vec<_>>(),
        }
    }
}

impl<T> From<(T, Option<T>, Option<T>)> for PotentialImportedBookmark
where
    T: AsRef<str>,
{
    fn from(tuple: (T, Option<T>, Option<T>)) -> Self {
        let (uri, title, tags) = tuple;
        Self {
            uri: uri.as_ref().to_string(),
            title: title.map(|t| t.as_ref().to_string()),
            tags: tags.map(|t| t.as_ref().to_string()),
        }
    }
}

impl<T> From<(T, Option<T>, Option<T>)> for PotentialBookmark
where
    T: AsRef<str>,
{
    fn from(tuple: (T, Option<T>, Option<T>)) -> Self {
        let (uri, title, tags) = tuple;
        Self {
            uri: uri.as_ref().to_string(),
            title: title.map(|t| t.as_ref().to_string()),
            tags: tags
                .map(|t| t.as_ref().to_string())
                .unwrap_or_default()
                .split(",")
                .map(|t| t.to_string())
                .collect::<Vec<_>>(),
        }
    }
}

impl<T> From<(T, Option<T>, &Vec<T>)> for PotentialBookmark
where
    T: AsRef<str>,
{
    fn from(tuple: (T, Option<T>, &Vec<T>)) -> Self {
        let (uri, title, tags) = tuple;
        Self {
            uri: uri.as_ref().to_string(),
            title: title.map(|t| t.as_ref().to_string()),
            tags: tags
                .iter()
                .map(|t| t.as_ref().to_string())
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DraftBookmarkError {
    #[error("couldn't parse provided uri value: {0}")]
    CouldntParseUri(ParseError),
    #[error("title is too long: {0} (max: {TAG_TITLE_MAX_LENGTH})")]
    TitleTooLong(usize),
    #[error("tags {0:?} are invalid (valid regex: {TAG_REGEX_STR})")]
    TagIsInvalid(Vec<String>),
}

#[derive(Debug)]
pub struct DraftBookmarkErrors {
    pub errors: Vec<(usize, DraftBookmarkError)>,
}

impl DraftBookmarkErrors {
    pub fn msg(&self) -> String {
        let num_errors = self.errors.len();
        if num_errors == 1 {
            "there was 1 validation error".into()
        } else {
            format!("there were {} validation errors", num_errors)
        }
    }
}

impl std::fmt::Display for DraftBookmarkErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let padding = match self.errors.last() {
            Some(e) => match e.0 {
                0 => 1,
                n => (n as f64).log10().floor() as usize + 1,
            },
            None => 1,
        };
        let num_errors = self.errors.len();
        for (i, (index, error)) in self.errors.iter().enumerate() {
            if i == num_errors - 1 {
                write!(
                    f,
                    "- entry {:width$}: {}",
                    index + 1,
                    error,
                    width = padding
                )?;
            } else {
                writeln!(
                    f,
                    "- entry {:width$}: {}",
                    index + 1,
                    error,
                    width = padding
                )?;
            }
        }

        Ok(())
    }
}

impl TryFrom<(PotentialBookmark, bool)> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: (PotentialBookmark, bool)) -> Result<Self, Self::Error> {
        #[allow(clippy::expect_used)]
        static WHITESPACE_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\s+").expect("regex is invalid"));

        let (potential_bookmark, ignore_attribute_errors) = value;
        let tags = &potential_bookmark.tags;

        Url::parse(&potential_bookmark.uri).map_err(DraftBookmarkError::CouldntParseUri)?;

        let title = match ignore_attribute_errors {
            true => potential_bookmark
                .title
                .as_ref()
                .map(|t| t.trim())
                .and_then(|t| {
                    if t.is_empty() {
                        None
                    } else if t.len() > TAG_TITLE_MAX_LENGTH {
                        t.get(0..TAG_TITLE_MAX_LENGTH)
                    } else {
                        Some(t)
                    }
                })
                .map(|t| t.to_string()),
            false => {
                if let Some(t) = &potential_bookmark.title {
                    let title_len = t.len();
                    if title_len > TAG_TITLE_MAX_LENGTH {
                        return Err(DraftBookmarkError::TitleTooLong(title_len));
                    }
                };

                potential_bookmark.title.as_ref().and_then(|t| {
                    let trimmed = t.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
            }
        };

        let tags = match ignore_attribute_errors {
            true => potential_bookmark
                .tags
                .iter()
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .map(|t| WHITESPACE_RE.replace_all(t, "-").to_string())
                .filter_map(|t| Tag::try_from(t.as_str()).ok())
                .collect::<Vec<_>>(),
            false => {
                let mut tags = Vec::with_capacity(tags.len());
                let mut invalid_tags = Vec::new();
                for tag in potential_bookmark.tags {
                    if tag.is_empty() {
                        continue;
                    }

                    match Tag::try_from(tag.as_str()) {
                        Ok(t) => tags.push(t),
                        Err(_) => invalid_tags.push(tag.to_string()),
                    }
                }
                if !invalid_tags.is_empty() {
                    return Err(DraftBookmarkError::TagIsInvalid(invalid_tags));
                }

                tags.sort();
                tags.dedup();
                tags
            }
        };

        Ok(Self {
            uri: potential_bookmark.uri,
            title,
            tags,
        })
    }
}

impl TryFrom<PotentialBookmark> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(potential_bookmark: PotentialBookmark) -> Result<Self, Self::Error> {
        Self::try_from((potential_bookmark, false))
    }
}

impl TryFrom<(PotentialImportedBookmark, bool)> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(tuple: (PotentialImportedBookmark, bool)) -> Result<Self, Self::Error> {
        let (potential_imported_bookmark, ignore_attribute_errors) = tuple;
        let potential_bookmark = PotentialBookmark::from(potential_imported_bookmark);
        Self::try_from((potential_bookmark, ignore_attribute_errors))
    }
}

impl TryFrom<PotentialImportedBookmark> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: PotentialImportedBookmark) -> Result<Self, Self::Error> {
        let potential_bookmark = PotentialBookmark::from(value);
        Self::try_from((potential_bookmark, false))
    }
}

impl DraftBookmark {
    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    pub fn title(&self) -> Option<&str> {
        match &self.title {
            Some(t) => Some(t.as_str()),
            None => None,
        }
    }

    pub fn tags(&self) -> Vec<&str> {
        self.tags.iter().map(|t| t.name()).collect()
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SavedBookmark {
    pub uri: String,
    pub title: Option<String>,
    pub tags: Option<String>,
}

#[allow(dead_code)]
fn dummy_fn(input: &str) -> String {
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn creating_a_draft_bookmark_works() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let tags = vec!["sql", "rust", "database-library-1"];
        let potential_bookmark = PotentialBookmark::from((uri, Some(title), &tags));

        // WHEN
        let result = DraftBookmark::try_from(potential_bookmark);

        // THEN
        assert!(result.is_ok());
    }

    #[test]
    fn empty_tags_get_skipped_over_while_creating_a_draft_bookmark() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let tags = vec!["sql", "", "database-library", ""];
        let potential_bookmark = PotentialBookmark::from((uri, Some(title), &tags));

        // WHEN
        let result = DraftBookmark::try_from(potential_bookmark)
            .expect("draft bookmark should've been created");

        // THEN
        assert_eq!(result.tags(), vec!["database-library", "sql"]);
    }

    #[test]
    fn force_creating_a_draft_bookmark_with_too_long_a_title_works() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "a".repeat(501);
        let tags = vec![];
        let potential_bookmark = PotentialBookmark::from((uri, Some(title.as_str()), &tags));

        // WHEN
        let draft_bookmark = DraftBookmark::try_from((potential_bookmark, true))
            .expect("draft bookmark should've been created");

        // THEN
        assert_eq!(
            draft_bookmark.title.map(|t| t.len()),
            Some(TAG_TITLE_MAX_LENGTH)
        );
    }

    #[test]
    fn force_creating_a_draft_bookmark_with_invalid_tags_works() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let tags = vec![
            "tag with spaces",
            "tag\twith\ttabs",
            "tag with trailing space   ",
            "  tag with leading space",
            "  tag with\t\tboth\ttabs and spaces  ",
            "inv@lid-t@g",
            "!!",
            "",
            "??",
        ];
        let potential_bookmark = PotentialBookmark::from((uri, None, &tags));

        // WHEN
        let draft_bookmark = DraftBookmark::try_from((potential_bookmark, true))
            .expect("draft bookmark should've been created");

        // THEN
        assert_eq!(
            draft_bookmark.tags(),
            vec![
                "tag-with-spaces",
                "tag-with-tabs",
                "tag-with-trailing-space",
                "tag-with-leading-space",
                "tag-with-both-tabs-and-spaces"
            ]
        );
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn draft_bookmark_cannot_be_created_with_an_incorrect_uri() {
        let faulty_uris = vec![
            "https:://github.com/launchbadge/sqlx",
            "github.com/launchbadge/sqlx",
            "https://github.com launchbadge/sqlx",
            "github",
        ];

        for uri in faulty_uris {
            // GIVEN
            let potential_bookmark = PotentialBookmark::from((uri, None, None));
            // WHEN
            let result = DraftBookmark::try_from(potential_bookmark);

            // THEN
            match result {
                Err(DraftBookmarkError::CouldntParseUri(_)) => (),
                _ => panic!("result is incorrect for {}", uri),
            }
        }
    }

    #[test]
    fn draft_bookmark_cannot_be_created_with_very_long_title() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "a".repeat(501);
        let potential_bookmark = PotentialBookmark::from((uri, Some(title.as_str()), None));

        // WHEN
        let result = DraftBookmark::try_from(potential_bookmark);

        // THEN
        match result {
            Err(DraftBookmarkError::TitleTooLong(_)) => (),
            _ => panic!("result is incorrect for {}", uri),
        }
    }

    #[test]
    fn draft_bookmark_cannot_be_created_with_an_malformed_tag() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let long_tag = "a".repeat(31);
        let malformed_tags = vec![
            "a tag with spaces",
            long_tag.as_str(),
            "^a [tag] with symbols $",
        ];

        for tag in malformed_tags {
            // WHEN
            let potential_bookmark = PotentialBookmark::from((uri, Some(title), &vec![tag]));
            let result = DraftBookmark::try_from(potential_bookmark);

            // THEN
            match result {
                Err(DraftBookmarkError::TagIsInvalid(_)) => (),
                _ => panic!("result is incorrect for {}", uri),
            }
        }
    }
}
