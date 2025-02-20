use super::tags::{Tag, TAG_REGEX_STR};
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
    pub tags: Option<String>,
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

impl TryFrom<(&str, Option<&str>, &Vec<&str>)> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: (&str, Option<&str>, &Vec<&str>)) -> Result<Self, Self::Error> {
        let (uri, title, tags) = value;

        Url::parse(uri).map_err(DraftBookmarkError::CouldntParseUri)?;

        if let Some(t) = title {
            let title_len = t.len();
            if title_len > TAG_TITLE_MAX_LENGTH {
                return Err(DraftBookmarkError::TitleTooLong(title_len));
            }
        };

        let title_to_save = title.and_then(|t| {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let mut tags_to_save = Vec::with_capacity(tags.len());
        let mut invalid_tags = Vec::new();
        for tag in tags {
            if tag.is_empty() {
                continue;
            }

            match Tag::try_from(tag) {
                Ok(t) => tags_to_save.push(t),
                Err(_) => invalid_tags.push(tag.to_string()),
            }
        }
        if !invalid_tags.is_empty() {
            return Err(DraftBookmarkError::TagIsInvalid(invalid_tags));
        }

        tags_to_save.sort();
        tags_to_save.dedup();

        Ok(Self {
            uri: uri.to_string(),
            title: title_to_save,
            tags: tags_to_save,
        })
    }
}

impl TryFrom<(&str, Option<&str>, Option<&str>)> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: (&str, Option<&str>, Option<&str>)) -> Result<Self, Self::Error> {
        let (uri, title, tags) = value;
        let tags = tags.map(|t| t.split(",").collect::<Vec<_>>());
        Self::try_from((uri, title, &tags.unwrap_or_default()))
    }
}

impl TryFrom<(&str, Option<&str>, Option<String>)> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: (&str, Option<&str>, Option<String>)) -> Result<Self, Self::Error> {
        let (uri, title, tags) = value;
        Self::try_from((uri, title, tags.as_deref()))
    }
}

impl TryFrom<(&str, Option<&str>, Option<Vec<String>>)> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: (&str, Option<&str>, Option<Vec<String>>)) -> Result<Self, Self::Error> {
        let (uri, title, tags) = value;
        let tags_ref = tags
            .as_ref()
            .map(|v| v.iter().map(|t| t.as_str()).collect::<Vec<_>>());
        Self::try_from((uri, title, &tags_ref.unwrap_or_default()))
    }
}

impl TryFrom<&String> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(uri: &String) -> Result<Self, Self::Error> {
        Self::try_from((uri.as_str(), None, &Vec::new()))
    }
}

impl TryFrom<&str> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        Self::try_from((uri, None, &Vec::new()))
    }
}

impl TryFrom<&PotentialBookmark> for DraftBookmark {
    type Error = DraftBookmarkError;

    fn try_from(value: &PotentialBookmark) -> Result<Self, Self::Error> {
        let t: Vec<&str> = value.tags.as_deref().unwrap_or("").split(",").collect();

        Self::try_from((value.uri.as_str(), value.title.as_deref(), &t))
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
    #[serde(skip)]
    #[allow(dead_code)]
    pub id: i64,
    pub uri: String,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub updated_at: i64,
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

        // WHEN
        let result = DraftBookmark::try_from((uri, Some(title), &tags));

        // THEN
        assert!(result.is_ok());
    }

    #[test]
    fn empty_tags_get_skipped_over_while_creating_a_draft_bookmark() {
        // GIVEN
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let tags = vec!["sql", "", "database-library", ""];

        // WHEN
        let result = DraftBookmark::try_from((uri, Some(title), &tags))
            .expect("draft bookmark should've been created");

        // THEN
        assert_eq!(result.tags(), vec!["database-library", "sql"]);
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn draft_bookmark_cannot_be_created_with_an_incorrect_uri() {
        // GIVEN
        let faulty_uris = vec![
            "https:://github.com/launchbadge/sqlx",
            "github.com/launchbadge/sqlx",
            "https://github.com launchbadge/sqlx",
            "github",
        ];

        for uri in faulty_uris {
            // WHEN
            let result = DraftBookmark::try_from((uri, None, &Vec::new()));

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

        // WHEN
        let result = DraftBookmark::try_from((uri, Some(title.as_str()), &Vec::new()));

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
            let result = DraftBookmark::try_from((uri, Some(title), &vec![tag]));

            // THEN
            match result {
                Err(DraftBookmarkError::TagIsInvalid(_)) => (),
                _ => panic!("result is incorrect for {}", uri),
            }
        }
    }
}
