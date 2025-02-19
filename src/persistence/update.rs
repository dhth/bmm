use super::errors::DBError;
use crate::domain::Tag;
use sqlx::{Pool, Sqlite};

pub async fn rename_tag_name(
    pool: &Pool<Sqlite>,
    source_tag: String,
    target_tag: Tag,
) -> Result<u64, DBError> {
    let new_tag_name = target_tag.name();

    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    let result = {
        let maybe_original_tag_id = sqlx::query!(
            "
SELECT
    id
FROM
    tags
WHERE
    name = ?
",
            source_tag
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("check if original tag exists".into(), e))?
        .map(|r| r.id);

        let original_tag_id = match maybe_original_tag_id {
            Some(id) => id,
            None => return Ok(0),
        };

        let maybe_new_tag_id = sqlx::query!(
            "
SELECT
    id
FROM
    tags
WHERE
    name = ?
",
            new_tag_name
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("check if new tag exists".into(), e))?
        .map(|r| r.id);

        match maybe_new_tag_id {
            Some(new_tag_id) => {
                sqlx::query!(
                    "
UPDATE
    bookmark_tags
SET
    tag_id = ?
WHERE
    tag_id = ?
AND NOT EXISTS (
    SELECT 1
    FROM bookmark_tags AS bt
    WHERE bt.bookmark_id = bookmark_tags.bookmark_id
    AND bt.tag_id = ?
)
",
                    new_tag_id,
                    original_tag_id,
                    new_tag_id,
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    DBError::CouldntExecuteQuery("replace tag id in bookmark_tags".to_string(), e)
                })?;

                sqlx::query!(
                    "
DELETE FROM
    tags
WHERE
    id = ?
",
                    original_tag_id,
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| DBError::CouldntExecuteQuery("delete original tag".to_string(), e))?
            }
            None => sqlx::query!(
                "
UPDATE
    tags
SET
    name = ?
WHERE
    id = ?
",
                new_tag_name,
                original_tag_id,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("rename tag".to_string(), e))?,
        }
    };

    tx.commit()
        .await
        .map_err(DBError::CouldntCommitTransaction)?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::domain::{DraftBookmark, Tag};
    use crate::persistence::test_fixtures::DBPoolFixture;
    use crate::persistence::{create_or_update_bookmark, get_bookmark_with_exact_uri, get_tags};

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[tokio::test]
    async fn renaming_tag_works_when_new_tag_doesnt_exist() {
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
    async fn renaming_tag_works_when_new_tag_already_exists() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let uris = [
            ("https://uri-one.com", None, vec!["tag1", "tag2"]),
            ("https://uri-two.com", None, vec!["tag2", "tag4"]),
            ("https://uri-three.com", None, vec!["tag1", "tag3"]),
            ("https://uri-four.com", None, vec!["tag1"]),
            ("https://uri-five.com", None, vec!["tag3"]),
        ];

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        for (uri, title, tags) in uris {
            let draft_bookmark = DraftBookmark::try_from((uri, title, tags))
                .expect("draft bookmark should be initialized");
            create_or_update_bookmark(&fixture.pool, &draft_bookmark, now)
                .await
                .expect("bookmark should be saved in db");
        }

        let new_tag = Tag::try_from("tag3").expect("new tag should've been created");
        let tags_before = get_tags(&fixture.pool)
            .await
            .expect("tags before should've been fetched");
        assert_eq!(tags_before.len(), 4, "tags before wasn't what was expected");

        // WHEN
        let rows_affected = rename_tag_name(&fixture.pool, "tag1".to_string(), new_tag)
            .await
            .expect("result should've been a success");

        // THEN
        assert_eq!(rows_affected, 1);

        let tags = get_tags(&fixture.pool)
            .await
            .expect("tags should've been fetched");
        assert_eq!(
            tags.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            vec!["tag2", "tag3", "tag4"]
        );

        let bookmark_one = get_bookmark_with_exact_uri(&fixture.pool, "https://uri-one.com")
            .await
            .expect("bookmark should've been fetched")
            .expect("bookmark should've been present");
        assert_eq!(bookmark_one.tags.as_deref(), Some("tag2,tag3"));

        let bookmark_two = get_bookmark_with_exact_uri(&fixture.pool, "https://uri-two.com")
            .await
            .expect("bookmark should've been fetched")
            .expect("bookmark should've been present");
        assert_eq!(bookmark_two.tags.as_deref(), Some("tag2,tag4"));

        let bookmark_three = get_bookmark_with_exact_uri(&fixture.pool, "https://uri-three.com")
            .await
            .expect("bookmark should've been fetched")
            .expect("bookmark should've been present");
        assert_eq!(bookmark_three.tags.as_deref(), Some("tag3"));

        let bookmark_four = get_bookmark_with_exact_uri(&fixture.pool, "https://uri-four.com")
            .await
            .expect("bookmark should've been fetched")
            .expect("bookmark should've been present");
        assert_eq!(bookmark_four.tags.as_deref(), Some("tag3"));

        let bookmark_five = get_bookmark_with_exact_uri(&fixture.pool, "https://uri-five.com")
            .await
            .expect("bookmark should've been fetched")
            .expect("bookmark should've been present");
        assert_eq!(bookmark_five.tags.as_deref(), Some("tag3"));
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
