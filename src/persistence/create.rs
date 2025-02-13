use super::errors::DBError;
use crate::domain::DraftBookmark;
use sqlx::Row;
use sqlx::{Pool, Sqlite};

pub async fn create_or_update_bookmark(
    pool: &Pool<Sqlite>,
    bookmark: &DraftBookmark,
    now: i64,
) -> Result<(), DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    {
        let uri = bookmark.uri();
        let title = bookmark.title();
        sqlx::query!(
            "
INSERT INTO
    bookmarks (uri, title, created_at, updated_at)
VALUES
    (?, ?, ?, ?) ON CONFLICT (uri) DO
UPDATE
SET
    title = excluded.title,
    updated_at = excluded.updated_at
",
            uri,
            title,
            now,
            now,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("insert bookmark".into(), e))?;

        let bookmark_id = sqlx::query!(
            "
SELECT
    id
FROM
    bookmarks
WHERE
    uri = ?
LIMIT 1
",
            uri
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("select bookmark id".into(), e))?
        .id;

        sqlx::query!(
            "
DELETE FROM
    bookmark_tags
WHERE
    bookmark_id = ?
",
            bookmark_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("delete old bookmark-tag pairs".into(), e))?;

        let tags = bookmark.tags();
        if !tags.is_empty() {
            for tag in &tags {
                sqlx::query!(
                    "
INSERT INTO
    tags (name)
VALUES
    (?) ON CONFLICT (name) DO NOTHING
",
                    *tag
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| DBError::CouldntExecuteQuery("upsert tags".into(), e))?;
            }

            let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
            let query = format!(
                "
SELECT
    id
FROM
    tags
WHERE
    name IN ({})
",
                placeholders.join(", ")
            );

            let mut query_builder = sqlx::query(&query);
            for name in tags {
                query_builder = query_builder.bind(name);
            }

            let rows = query_builder
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| DBError::CouldntExecuteQuery("fetch tag ids".into(), e))?;

            let mut tag_ids: Vec<i64> = Vec::new();
            for row in rows {
                let id: i64 = row.try_get("id").map_err(DBError::CouldntConvertFromSQL)?;
                tag_ids.push(id);
            }

            for tag_id in tag_ids {
                sqlx::query!(
                    "
INSERT INTO
    bookmark_tags (bookmark_id, tag_id)
VALUES
    (?, ?) ON CONFLICT (bookmark_id, tag_id) DO NOTHING
",
                    bookmark_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| DBError::CouldntExecuteQuery("insert bookmark-tag pair".into(), e))?;
            }
        }

        // clean up of unused tags
        sqlx::query!(
            "
DELETE FROM
    tags
WHERE
    id NOT IN (
        SELECT
            tag_id
        FROM
            bookmark_tags
    )
",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("clean up unused tags".into(), e))?;
    }

    tx.commit()
        .await
        .map_err(DBError::CouldntCommitTransaction)?;

    Ok(())
}

pub async fn create_or_update_bookmarks(
    pool: &Pool<Sqlite>,
    bookmarks: &Vec<DraftBookmark>,
    now: i64,
) -> Result<(), DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    {
        for bookmark in bookmarks {
            let uri = bookmark.uri();
            let title = bookmark.title();
            sqlx::query!(
                "
INSERT INTO
    bookmarks (uri, title, created_at, updated_at)
VALUES
    (?, ?, ?, ?) ON CONFLICT (uri) DO
UPDATE
SET
    title = excluded.title,
    updated_at = excluded.updated_at
",
                uri,
                title,
                now,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("insert bookmark".into(), e))?;

            let bookmark_id = sqlx::query!(
                "
SELECT
    id
FROM
    bookmarks
WHERE
    uri = ?
LIMIT 1
",
                uri
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("select bookmark id".into(), e))?
            .id;

            sqlx::query!(
                "
DELETE FROM
    bookmark_tags
WHERE
    bookmark_id = ?
",
                bookmark_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("delete old bookmark-tag pairs".into(), e))?;

            let tags = bookmark.tags();
            if !tags.is_empty() {
                for tag in &tags {
                    sqlx::query!(
                        "
INSERT INTO
    tags (name)
VALUES
    (?) ON CONFLICT (name) DO NOTHING
",
                        *tag
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| DBError::CouldntExecuteQuery("upsert tags".into(), e))?;
                }

                let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
                let query = format!(
                    "
SELECT
    id
FROM
    tags
WHERE
    name IN ({})
",
                    placeholders.join(", ")
                );

                let mut query_builder = sqlx::query(&query);
                for name in tags {
                    query_builder = query_builder.bind(name);
                }

                let rows = query_builder
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(|e| DBError::CouldntExecuteQuery("fetch tag ids".into(), e))?;

                let mut tag_ids: Vec<i64> = Vec::new();
                for row in rows {
                    let id: i64 = row.try_get("id").map_err(DBError::CouldntConvertFromSQL)?;
                    tag_ids.push(id);
                }

                for tag_id in tag_ids {
                    sqlx::query!(
                        "
INSERT INTO
    bookmark_tags (bookmark_id, tag_id)
VALUES
    (?, ?) ON CONFLICT (bookmark_id, tag_id) DO NOTHING
",
                        bookmark_id,
                        tag_id
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        DBError::CouldntExecuteQuery("insert bookmark-tag pair".into(), e)
                    })?;
                }
            }
        }

        // clean up of unused tags
        sqlx::query!(
            "
DELETE FROM
    tags
WHERE
    id NOT IN (
        SELECT
            tag_id
        FROM
            bookmark_tags
    )
",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("clean up unused tags".into(), e))?;
    }

    tx.commit()
        .await
        .map_err(DBError::CouldntCommitTransaction)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::get::{
        get_bookmark_tags, get_bookmark_with_exact_uri, get_num_bookmarks, get_tags,
    };
    use super::super::init::get_in_memory_db_pool;
    use super::*;
    use pretty_assertions::assert_eq;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Fixture {
        pool: Pool<Sqlite>,
    }

    impl Fixture {
        async fn new() -> Self {
            let pool = get_in_memory_db_pool()
                .await
                .expect("in memory connection pool should've been created");
            Self { pool }
        }
    }

    #[tokio::test]
    async fn creating_a_bookmark_with_all_attributes_works() {
        // GIVEN
        let fixture = Fixture::new().await;
        let tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark = DraftBookmark::try_from((uri, Some(title), tags))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        // WHEN
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let saved_bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .expect("should have queried bookmark")
            .expect("queried result should've contained a bookmark");
        assert_eq!(saved_bookmark.title, Some(title.into()));

        let saved_tags = get_bookmark_tags(&fixture.pool, saved_bookmark.id, 10)
            .await
            .expect("should have queried tags");
        assert_eq!(saved_tags.len(), 2, "number of tags is incorrect");
        assert!(
            saved_tags.contains(&"rust".to_string()),
            "tag1 not found in the db"
        );
        assert!(
            saved_tags.contains(&"sqlite".to_string()),
            "tag2 not found in the db"
        );
    }

    #[tokio::test]
    async fn creating_a_bookmark_without_a_title_works() {
        // GIVEN
        let fixture = Fixture::new().await;
        let tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let draft_bookmark = DraftBookmark::try_from((uri, None, tags))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        // WHEN
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let saved_bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .expect("should have queried bookmark")
            .expect("queried result should've contained a bookmark");
        assert!(saved_bookmark.title.is_none());

        let saved_tags = get_bookmark_tags(&fixture.pool, saved_bookmark.id, 10)
            .await
            .expect("should have queried tags");
        assert_eq!(saved_tags.len(), 2, "number of tags is incorrect");
        assert!(
            saved_tags.contains(&"rust".to_string()),
            "tag1 not found in the db"
        );
        assert!(
            saved_tags.contains(&"sqlite".to_string()),
            "tag2 not found in the db"
        );
    }

    #[tokio::test]
    async fn creating_a_bookmark_without_tags_works() {
        // GIVEN
        let fixture = Fixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark = DraftBookmark::try_from((uri, Some(title), vec![]))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        // WHEN
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let saved_bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .expect("should have queried bookmark")
            .expect("queried result should've contained a bookmark");
        assert_eq!(saved_bookmark.title, Some(title.into()));

        let saved_tags = get_bookmark_tags(&fixture.pool, saved_bookmark.id, 10)
            .await
            .expect("should have queried tags");
        assert_eq!(saved_tags.len(), 0, "number of tags is incorrect");
    }

    #[tokio::test]
    async fn updating_a_bookmark_overwrites_previous_data() {
        // GIVEN
        let fixture = Fixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old = DraftBookmark::try_from((uri, Some(title_old), old_tags))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(&fixture.pool, &draft_bookmark_old, created_at)
            .await
            .expect("bookmark should've been saved the first time");

        // WHEN
        let new_tags = vec!["rust", "github", "database"];
        let title_new = "sqlx's github repository";
        let draft_bookmark = DraftBookmark::try_from((uri, Some(title_new), new_tags))
            .expect("draft bookmark should've been created");
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let num_bookmarks = get_num_bookmarks(&fixture.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert_eq!(saved_bookmark.title, Some(title_new.into()));
        assert_eq!(saved_bookmark.updated_at, now);

        let saved_tags = get_bookmark_tags(&fixture.pool, saved_bookmark.id, 10)
            .await
            .expect("should have queried tags");
        assert_eq!(
            saved_tags.len(),
            3,
            "total saved tags for bookmark were not 4"
        );
        assert!(
            saved_tags.contains(&"rust".to_string()),
            "bookmark doesn't have tag1"
        );
        assert!(
            saved_tags.contains(&"github".to_string()),
            "bookmark doesn't have tag2"
        );
        assert!(
            saved_tags.contains(&"database".to_string()),
            "bookmark doesn't have tag3"
        );
    }

    #[tokio::test]
    async fn removing_title_from_a_saved_bookmark_works() {
        // GIVEN
        let fixture = Fixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old = DraftBookmark::try_from((uri, Some(title_old), vec![]))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(&fixture.pool, &draft_bookmark_old, created_at)
            .await
            .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark = DraftBookmark::try_from((uri, None, vec![]))
            .expect("draft bookmark should've been created");
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let saved_bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert!(saved_bookmark.title.is_none());
        assert_eq!(saved_bookmark.updated_at, now);
    }

    #[tokio::test]
    async fn removing_tags_from_a_saved_bookmark_works() {
        // GIVEN
        let fixture = Fixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark_old = DraftBookmark::try_from((uri, Some(title), old_tags))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(&fixture.pool, &draft_bookmark_old, created_at)
            .await
            .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark = DraftBookmark::try_from((uri, Some(title), vec![]))
            .expect("draft bookmark should've been created");
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let num_bookmarks = get_num_bookmarks(&fixture.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert_eq!(saved_bookmark.updated_at, now);

        let saved_tags = get_bookmark_tags(&fixture.pool, saved_bookmark.id, 10)
            .await
            .expect("should have queried tags");
        assert_eq!(
            saved_tags.len(),
            0,
            "total saved tags for bookmark were not 0"
        );
    }

    #[tokio::test]
    async fn updating_bookmark_cleans_up_unused_tags() {
        // GIVEN
        let fixture = Fixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark_old = DraftBookmark::try_from((uri, Some(title), old_tags))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(&fixture.pool, &draft_bookmark_old, created_at)
            .await
            .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark = DraftBookmark::try_from((uri, Some(title), vec![]))
            .expect("draft bookmark should've been created");
        let result = create_or_update_bookmark(&fixture.pool, &draft_bookmark, now).await;

        // THEN
        if let Err(error) = &result {
            println!("error: {}", error);
        }
        assert!(result.is_ok(), "result is not ok");

        let all_tags = get_tags(&fixture.pool)
            .await
            .expect("should have queried all tags");

        assert_eq!(
            all_tags.len(),
            0,
            "total number of tags in the db should've been zero"
        );
    }
}
