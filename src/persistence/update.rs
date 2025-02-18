use super::errors::DBError;
use crate::domain::Tag;
use sqlx::{Pool, Sqlite};

pub async fn rename_tag_name(
    pool: &Pool<Sqlite>,
    original_tag: String,
    new_tag: Tag,
) -> Result<u64, DBError> {
    let new_tag_value = new_tag.value();
    let result = sqlx::query!(
        "
UPDATE
    tags
SET
    name = ?
WHERE
    name = ?
",
        new_tag_value,
        original_tag,
    )
    .execute(pool)
    .await
    .map_err(|e| DBError::CouldntExecuteQuery("rename tag".to_string(), e))?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::domain::{DraftBookmark, Tag};
    use crate::persistence::test_fixtures::DBPoolFixture;
    use crate::persistence::{create_or_update_bookmark, get_tags};

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[tokio::test]
    async fn renaming_tag_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let draft_bookmark = DraftBookmark::try_from((uri, None, vec!["old-tag-1", "old-tag-2"]))
            .expect("draft bookmark should be initialized");

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        create_or_update_bookmark(&fixture.pool, &draft_bookmark, now)
            .await
            .expect("bookmark should be saved in db");

        let new_tag = Tag::try_from("new-tag").expect("new tag should've been created");

        // WHEN
        let rows_affected = rename_tag_name(&fixture.pool, "old-tag-1".to_string(), new_tag)
            .await
            .expect("result should've been a success");

        // THEN
        assert_eq!(rows_affected, 1);

        let tags = get_tags(&fixture.pool)
            .await
            .expect("tags should've been fetched");
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"new-tag".to_string()));
        assert!(tags.contains(&"old-tag-2".to_string()));
    }

    #[tokio::test]
    async fn renaming_non_existent_tag_doesnt_fail() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let draft_bookmark = DraftBookmark::try_from((uri, None, vec!["old-tag-1", "old-tag-2"]))
            .expect("draft bookmark should be initialized");

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        create_or_update_bookmark(&fixture.pool, &draft_bookmark, now)
            .await
            .expect("bookmark should be saved in db");
        let new_tag = Tag::try_from("new-tag").expect("new tag should've been created");

        // WHEN
        let rows_affected = rename_tag_name(&fixture.pool, "old-tag-3".to_string(), new_tag)
            .await
            .expect("result should've been a success");

        // THEN
        assert_eq!(rows_affected, 0);
    }
}

