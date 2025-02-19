use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

pub const TAG_REGEX_STR: &str = r"^[a-zA-Z0-9_-]{1,30}$";

#[derive(PartialEq, Eq, Serialize, Debug, PartialOrd, Ord)]
pub struct Tag(String);

impl Tag {
    pub fn name(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<&&str> for Tag {
    type Error = ();

    #[allow(clippy::expect_used)]
    fn try_from(tag: &&str) -> Result<Self, Self::Error> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(TAG_REGEX_STR).expect("regex is invalid"));

        let trimmed_tag = tag.trim();
        if trimmed_tag.is_empty() {
            return Err(());
        }
        if !RE.is_match(trimmed_tag) {
            return Err(());
        }

        Ok(Self(trimmed_tag.to_lowercase().to_string()))
    }
}

impl TryFrom<&str> for Tag {
    type Error = ();

    fn try_from(tag: &str) -> Result<Self, Self::Error> {
        Self::try_from(&tag)
    }
}

#[derive(Debug, Serialize)]
pub struct TagStats {
    pub name: String,
    pub num_bookmarks: i64,
}

impl std::fmt::Display for TagStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} bookmarks)", self.name, self.num_bookmarks)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn parsing_valid_tag_works() {
        let valid_tags = vec!["tag", "tAg", "tag1", "t1ag2", "tag-1", "tag_1"];
        for tag in valid_tags {
            // GIVEN
            // WHEN
            let result = Tag::try_from(tag);

            // THEN
            assert!(result.is_ok())
        }
    }

    #[test]
    fn tags_get_trimmed_during_parsing() {
        // GIVEN
        // WHEN
        let result = Tag::try_from("  a-tag-with-spaces-at-each-end  ")
            .expect("result should've been a success");

        // THEN
        assert_eq!(result.name(), "a-tag-with-spaces-at-each-end");
    }

    #[test]
    fn tags_get_converted_to_lowercase_during_parsing() {
        // GIVEN
        // WHEN
        let result =
            Tag::try_from("UPPER-and-lower-case-chars").expect("result should've been a success");
        // THEN
        assert_eq!(result.name(), "upper-and-lower-case-chars");
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn parsing_invalid_tag_fails() {
        let invalid_tags = vec!["", "t ag", "tag??", "ta!g", "[tag]", "tag$"];
        for tag in invalid_tags {
            // GIVEN
            // WHEN
            let result = Tag::try_from(tag);

            // THEN
            assert!(result.is_err())
        }
    }
}
