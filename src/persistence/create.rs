use super::errors::DBError;
use crate::domain::DraftBookmark;
use sqlx::Row;
use sqlx::{Pool, Sqlite};

#[derive(Clone, Copy, Default)]
pub struct SaveBookmarkOptions {
    pub reset_missing_attributes: bool,
    pub reset_tags: bool,
}

pub async fn create_or_update_bookmark(
    pool: &Pool<Sqlite>,
    bookmark: &DraftBookmark,
    now: i64,
    options: SaveBookmarkOptions,
) -> Result<(), DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    {
        let uri = bookmark.uri();
        let title = bookmark.title();
        match options.reset_missing_attributes {
            true => {
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
            }

            false => {
                sqlx::query!(
                    "
INSERT INTO
    bookmarks (uri, title, created_at, updated_at)
VALUES
    (?, ?, ?, ?) ON CONFLICT (uri) DO
UPDATE
SET
    title = COALESCE(excluded.title, bookmarks.title),
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
            }
        }

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

        if options.reset_tags {
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
        }

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
    options: SaveBookmarkOptions,
) -> Result<(), DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    {
        for bookmark in bookmarks {
            let uri = bookmark.uri();
            let title = bookmark.title();
            match options.reset_missing_attributes {
                true => {
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
                }
                false => {
                    sqlx::query!(
                        "
INSERT INTO
    bookmarks (uri, title, created_at, updated_at)
VALUES
    (?, ?, ?, ?) ON CONFLICT (uri) DO
UPDATE
SET
    title = COALESCE(excluded.title, bookmarks.title),
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
                }
            }

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

            if options.reset_tags {
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
                .map_err(|e| {
                    DBError::CouldntExecuteQuery("delete old bookmark-tag pairs".into(), e)
                })?;
            }

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
        get_all_bookmarks, get_bookmark_with_exact_uri, get_num_bookmarks, get_tags,
    };
    use super::super::test_fixtures::DBPoolFixture;
    use super::*;
    use crate::domain::PotentialBookmark;
    use insta::assert_yaml_snapshot;

    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn creating_a_bookmark_with_all_attributes_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), &tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        // WHEN
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been created");

        // THEN
        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("should have queried bookmark")
            .expect("queried result should've contained a bookmark");
        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: "rust,sqlite"
        "#);
    }

    #[tokio::test]
    async fn creating_a_bookmark_without_a_title_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let draft_bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, None, &tags)))
            .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        // WHEN
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been created");

        // THEN
        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("should have queried bookmark")
            .expect("queried result should've contained a bookmark");
        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: ~
        tags: "rust,sqlite"
        "#);
    }

    #[tokio::test]
    async fn creating_a_bookmark_without_tags_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), &vec![])))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        // WHEN
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been created");

        // THEN
        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("should have queried bookmark")
            .expect("queried result should've contained a bookmark");
        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: ~
        "#);
    }

    #[tokio::test]
    async fn updating_a_bookmark_keeps_previous_data_if_requested() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_old), &old_tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, None, None)))
            .expect("draft bookmark should've been created");
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been updated");

        // THEN
        let num_bookmarks = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: "rust,sqlite"
        "#);
    }

    #[tokio::test]
    async fn updating_a_bookmark_appends_to_previous_tags_if_requested() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_old), &old_tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let new_tags = vec!["rust", "github", "database"];
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, None, &new_tags)))
                .expect("draft bookmark should've been created");
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been updated");

        // THEN
        let num_bookmarks = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: "database,github,rust,sqlite"
        "#);

        let tags = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags, @r"
        - database
        - github
        - rust
        - sqlite
        ");
    }

    #[tokio::test]
    async fn updating_a_bookmark_overwrites_previous_attributes_if_requested() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_old), &old_tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let title_new = "sqlx's github repository";
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_new), &old_tags)))
                .expect("draft bookmark should've been created");
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: true,
            reset_tags: true,
        };
        create_or_update_bookmark(&fx.pool, &draft_bookmark, now, save_options)
            .await
            .expect("bookmark should've been updated");

        // THEN
        let num_bookmarks = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github repository"
        tags: "rust,sqlite"
        "#);
    }

    #[tokio::test]
    async fn updating_a_bookmark_overwrites_previous_tags_if_requested() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_old), &old_tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let new_tags = vec!["rust", "github", "database"];
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_old), &new_tags)))
                .expect("draft bookmark should've been created");
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: false,
            reset_tags: true,
        };
        create_or_update_bookmark(&fx.pool, &draft_bookmark, now, save_options)
            .await
            .expect("bookmark should've been updated");

        // THEN
        let num_bookmarks = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");
        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: "database,github,rust"
        "#);
    }

    #[tokio::test]
    async fn removing_title_from_a_saved_bookmark_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let title_old = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title_old), &vec![])))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, None, &vec![])))
            .expect("draft bookmark should've been created");
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: true,
            reset_tags: false,
        };
        create_or_update_bookmark(&fx.pool, &draft_bookmark, now, save_options)
            .await
            .expect("bookmark should've been updated");

        // THEN
        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");

        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: ~
        tags: ~
        "#);
    }

    #[tokio::test]
    async fn removing_tags_from_a_saved_bookmark_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), &old_tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), &vec![])))
                .expect("draft bookmark should've been created");
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: false,
            reset_tags: true,
        };
        create_or_update_bookmark(&fx.pool, &draft_bookmark, now, save_options)
            .await
            .expect("bookmark should've been updated");

        // THEN
        let num_bookmarks = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks, 1);

        let saved_bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
            .await
            .expect("bookmark should've been queried")
            .expect("queried result should've contained a bookmark");
        assert_yaml_snapshot!(saved_bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: ~
        "#);
    }

    #[tokio::test]
    async fn updating_bookmark_cleans_up_unused_tags() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let old_tags = vec!["rust", "sqlite"];
        let uri = "https://github.com/launchbadge/sqlx";
        let title = "sqlx's github page";
        let draft_bookmark_old =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), &old_tags)))
                .expect("draft bookmark should've been created");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let created_at = now - 60 * 60;

        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark_old,
            created_at,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved the first time");

        // WHEN
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), &vec![])))
                .expect("draft bookmark should've been created");
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: false,
            reset_tags: true,
        };
        create_or_update_bookmark(&fx.pool, &draft_bookmark, now, save_options)
            .await
            .expect("bookmark should've been updated");

        // THEN
        let all_tags = get_tags(&fx.pool)
            .await
            .expect("should have queried all tags");

        assert_yaml_snapshot!(all_tags, @"[]");
    }

    #[tokio::test]
    async fn creating_multiple_bookmarks_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uris = [
            ("https://uri-one.com", None, vec!["tag5", "tag2"]),
            ("https://uri-two.com", None, vec!["tag2", "tag3"]),
            ("https://uri-three.com", None, vec!["tag2", "tag3"]),
            ("https://uri-four.com", None, vec!["tag1", "tag3"]),
            ("https://uri-five.com", None, vec!["tag3", "tag4"]),
        ];

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let draft_bookmarks = uris
            .into_iter()
            .map(|(uri, title, tags)| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        // WHEN
        create_or_update_bookmarks(
            &fx.pool,
            &draft_bookmarks,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmarks should've been created");

        // THEN
        let bookmarks = get_all_bookmarks(&fx.pool)
            .await
            .expect("bookmarks should've been fetched");
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://uri-one.com"
          title: ~
          tags: "tag2,tag5"
        - uri: "https://uri-two.com"
          title: ~
          tags: "tag2,tag3"
        - uri: "https://uri-three.com"
          title: ~
          tags: "tag2,tag3"
        - uri: "https://uri-four.com"
          title: ~
          tags: "tag1,tag3"
        - uri: "https://uri-five.com"
          title: ~
          tags: "tag3,tag4"
        "#);

        let tags = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags, @r"
        - tag1
        - tag2
        - tag3
        - tag4
        - tag5
        ");
    }

    #[tokio::test]
    async fn updating_multiple_bookmarks_without_resetting_original_details_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uris = [
            ("https://uri-one.com", Some("title"), vec!["tag5", "tag2"]),
            ("https://uri-two.com", None, vec!["tag2", "tag3"]),
            ("https://uri-three.com", None, vec!["tag2", "tag3"]),
            ("https://uri-four.com", None, vec!["tag1", "tag3"]),
            ("https://uri-five.com", None, vec!["tag3", "tag4"]),
        ];

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let draft_bookmarks_original = uris
            .into_iter()
            .map(|(uri, title, tags)| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        create_or_update_bookmarks(
            &fx.pool,
            &draft_bookmarks_original,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmarks should've been created");

        let updated_uris: [(&str, Option<&str>, Vec<&str>); 6] = [
            ("https://uri-one.com", None, vec![]),
            ("https://uri-two.com", None, vec![]),
            ("https://uri-three.com", None, vec!["tag3"]),
            ("https://uri-four.com", None, vec!["tag1", "tag3"]),
            ("https://uri-five.com", None, vec!["tag3", "tag4"]),
            ("https://uri-six.com", None, vec!["tag6", "tag7"]),
        ];

        let draft_bookmarks = updated_uris
            .into_iter()
            .map(|(uri, title, tags)| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        // WHEN

        create_or_update_bookmarks(
            &fx.pool,
            &draft_bookmarks,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmarks should've been updated");

        // THEN
        let bookmarks = get_all_bookmarks(&fx.pool)
            .await
            .expect("bookmarks should've been fetched");
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://uri-one.com"
          title: title
          tags: "tag2,tag5"
        - uri: "https://uri-two.com"
          title: ~
          tags: "tag2,tag3"
        - uri: "https://uri-three.com"
          title: ~
          tags: "tag2,tag3"
        - uri: "https://uri-four.com"
          title: ~
          tags: "tag1,tag3"
        - uri: "https://uri-five.com"
          title: ~
          tags: "tag3,tag4"
        - uri: "https://uri-six.com"
          title: ~
          tags: "tag6,tag7"
        "#);

        let tags = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags, @r"
        - tag1
        - tag2
        - tag3
        - tag4
        - tag5
        - tag6
        - tag7
        ");
    }

    #[tokio::test]
    async fn resetting_attributes_for_multiple_bookmarks_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uris = [
            ("https://uri-one.com", Some("title"), vec!["tag5", "tag2"]),
            ("https://uri-two.com", None, vec!["tag2", "tag3"]),
            ("https://uri-three.com", None, vec!["tag2", "tag3"]),
            ("https://uri-four.com", None, vec!["tag1", "tag3"]),
            ("https://uri-five.com", None, vec!["tag3", "tag4"]),
        ];

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let draft_bookmarks_original = uris
            .into_iter()
            .map(|(uri, title, tags)| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        create_or_update_bookmarks(
            &fx.pool,
            &draft_bookmarks_original,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmarks should've been created");

        let updated_uris = [
            "https://uri-one.com",
            "https://uri-two.com",
            "https://uri-three.com",
            "https://uri-four.com",
            "https://uri-five.com",
            "https://uri-six.com",
        ];

        let draft_bookmarks = updated_uris
            .into_iter()
            .map(|uri| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, None, None)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        // WHEN
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: true,
            reset_tags: false,
        };
        create_or_update_bookmarks(&fx.pool, &draft_bookmarks, now, save_options)
            .await
            .expect("bookmarks should've been updated");

        // THEN
        let bookmarks = get_all_bookmarks(&fx.pool)
            .await
            .expect("bookmarks should've been fetched");
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://uri-one.com"
          title: ~
          tags: "tag2,tag5"
        - uri: "https://uri-two.com"
          title: ~
          tags: "tag2,tag3"
        - uri: "https://uri-three.com"
          title: ~
          tags: "tag2,tag3"
        - uri: "https://uri-four.com"
          title: ~
          tags: "tag1,tag3"
        - uri: "https://uri-five.com"
          title: ~
          tags: "tag3,tag4"
        - uri: "https://uri-six.com"
          title: ~
          tags: ~
        "#);

        let tags = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags, @r"
        - tag1
        - tag2
        - tag3
        - tag4
        - tag5
        ");
    }

    #[tokio::test]
    async fn resetting_tags_for_multiple_bookmarks_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uris = [
            ("https://uri-one.com", Some("title"), vec!["tag5", "tag2"]),
            ("https://uri-two.com", None, vec!["tag2", "tag3"]),
            ("https://uri-three.com", None, vec!["tag2", "tag3"]),
            ("https://uri-four.com", None, vec!["tag1", "tag3"]),
            ("https://uri-five.com", None, vec!["tag3", "tag4"]),
        ];

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let draft_bookmarks_original = uris
            .into_iter()
            .map(|(uri, title, tags)| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        create_or_update_bookmarks(
            &fx.pool,
            &draft_bookmarks_original,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmarks should've been created");

        let updated_uris = [
            "https://uri-one.com",
            "https://uri-two.com",
            "https://uri-three.com",
            "https://uri-four.com",
            "https://uri-five.com",
            "https://uri-six.com",
        ];

        let draft_bookmarks = updated_uris
            .into_iter()
            .map(|uri| {
                DraftBookmark::try_from(PotentialBookmark::from((uri, None, None)))
                    .expect("draft bookmarks should've been initialized")
            })
            .collect::<Vec<_>>();

        // WHEN
        let save_options = SaveBookmarkOptions {
            reset_missing_attributes: false,
            reset_tags: true,
        };
        create_or_update_bookmarks(&fx.pool, &draft_bookmarks, now, save_options)
            .await
            .expect("bookmarks should've been updated");

        // THEN
        let bookmarks = get_all_bookmarks(&fx.pool)
            .await
            .expect("bookmarks should've been fetched");
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://uri-one.com"
          title: title
          tags: ~
        - uri: "https://uri-two.com"
          title: ~
          tags: ~
        - uri: "https://uri-three.com"
          title: ~
          tags: ~
        - uri: "https://uri-four.com"
          title: ~
          tags: ~
        - uri: "https://uri-five.com"
          title: ~
          tags: ~
        - uri: "https://uri-six.com"
          title: ~
          tags: ~
        "#);

        let tags = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags, @"[]");
    }
}
