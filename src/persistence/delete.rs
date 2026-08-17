use super::DBError;
use sqlx::{Pool, Sqlite};

const DELETE_BATCH_SIZE: usize = 999;

pub async fn delete_bookmarks_with_uris(
    pool: &Pool<Sqlite>,
    uris: &[String],
) -> Result<u64, DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    let mut rows_affected = 0;

    for uri_chunk in uris.chunks(DELETE_BATCH_SIZE) {
        let query = format!(
            r#"
DELETE FROM
    bookmarks
WHERE
    id IN (
        SELECT
            id
        FROM
            bookmarks
        WHERE
            uri IN ({})
    )
"#,
            uri_chunk
                .iter()
                .map(|_| "?")
                .collect::<Vec<&str>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query::<_>(&query);
        for uri in uri_chunk {
            query_builder = query_builder.bind(uri.as_str());
        }

        let result = query_builder
            .execute(&mut *tx)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("delete bookmarks with uris".into(), e))?;

        rows_affected += result.rows_affected();
    }

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

    tx.commit()
        .await
        .map_err(DBError::CouldntCommitTransaction)?;

    Ok(rows_affected)
}

pub async fn delete_tags_by_name(pool: &Pool<Sqlite>, tags: &[String]) -> Result<u64, DBError> {
    let query = format!(
        r#"
DELETE FROM
    tags
WHERE
    name IN ({})
"#,
        tags.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
    );
    let mut query_builder = sqlx::query(&query);
    for tag in tags {
        query_builder = query_builder.bind(tag);
    }

    let result = query_builder
        .execute(pool)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("delete tags".into(), e))?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::super::test_fixtures::DBPoolFixture;
    use super::super::{
        create_or_update_bookmark, create_or_update_bookmarks, get_all_bookmarks,
        get_num_bookmarks, get_tags,
    };
    use super::*;
    use crate::domain::{DraftBookmark, PotentialBookmark};
    use crate::persistence::SaveBookmarkOptions;
    use insta::assert_yaml_snapshot;

    use std::time::{SystemTime, UNIX_EPOCH};

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[tokio::test]
    async fn deleting_uris_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let num_bookmarks = 10;

        for i in 1..=num_bookmarks {
            let uri = format!("https://uri-{i}.com");
            let draft_bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri.as_str(), None, &vec![])))
                    .expect("draft bookmark should've been created");
            println!("draft_bookmark: {}", draft_bookmark.uri());
            create_or_update_bookmark(
                &fx.pool,
                &draft_bookmark,
                now,
                SaveBookmarkOptions::default(),
            )
            .await
            .expect("bookmark should've been saved in db");
        }

        // WHEN
        let uris_to_delete = vec!["https://uri-1.com".into(), "https://uri-4.com".into()];

        let result = delete_bookmarks_with_uris(&fx.pool, &uris_to_delete)
            .await
            .expect("result should've been a success");
        assert_eq!(result, uris_to_delete.len() as u64);

        let num_bookmarks_in_db = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(
            num_bookmarks_in_db,
            num_bookmarks - uris_to_delete.len() as i64
        );
    }

    #[tokio::test]
    async fn deleting_uris_works_when_uris_dont_exist() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let uri = "https://uri.com";
        let draft_bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, None, &vec![])))
            .expect("draft bookmark should've been created");
        println!("draft_bookmark: {}", draft_bookmark.uri());
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved in db");

        // WHEN
        let uris_to_delete = vec![
            "https://unknown-uri-1.com".into(),
            "https://unknown-uri-2.com".into(),
        ];

        let result = delete_bookmarks_with_uris(&fx.pool, &uris_to_delete)
            .await
            .expect("result should've been a success");
        assert_eq!(result, 0);

        let num_bookmarks_in_db = get_num_bookmarks(&fx.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks_in_db, 1);
    }

    #[tokio::test]
    async fn deleting_uris_cleans_up_unused_tags() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let uri = "https://uri.com";
        let draft_bookmark =
            DraftBookmark::try_from(PotentialBookmark::from((uri, None, &vec!["tag"])))
                .expect("draft bookmark should've been created");
        println!("draft_bookmark: {}", draft_bookmark.uri());
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should've been saved in db");

        // WHEN
        let uris_to_delete = vec![uri.to_string()];

        let result = delete_bookmarks_with_uris(&fx.pool, &uris_to_delete)
            .await
            .expect("result should've been a success");
        assert_eq!(result, 1);

        let tags_in_db = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags_in_db, @"[]");
    }

    #[tokio::test]
    async fn deleting_more_uris_than_the_batch_size_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("system time should've been after the Unix epoch");
        let now = since_the_epoch.as_secs() as i64;
        let num_bookmarks_to_delete = DELETE_BATCH_SIZE * 2 + 1;
        let mut uris_to_delete = Vec::with_capacity(num_bookmarks_to_delete);
        let mut draft_bookmarks = Vec::with_capacity(num_bookmarks_to_delete + 2);

        for i in 0..num_bookmarks_to_delete {
            let uri = format!("https://uri-{i}.com");
            let tags = if i % DELETE_BATCH_SIZE == 0 {
                vec!["deleted-only", "shared"]
            } else {
                vec![]
            };
            let draft_bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri.as_str(), None, &tags)))
                    .expect("draft bookmark should've been created");
            draft_bookmarks.push(draft_bookmark);
            uris_to_delete.push(uri);
        }

        for (uri, title, tags) in [
            (
                "https://retained-one.com",
                Some("retained bookmark"),
                vec!["retained-only", "shared"],
            ),
            ("https://retained-two.com", None, vec![]),
        ] {
            let draft_bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmark should've been created");
            draft_bookmarks.push(draft_bookmark);
        }

        create_or_update_bookmarks(
            &fx.pool,
            &draft_bookmarks,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmarks should've been saved in db");

        // WHEN
        let rows_affected = delete_bookmarks_with_uris(&fx.pool, &uris_to_delete)
            .await
            .expect("bookmarks should've been deleted");

        // THEN
        assert_eq!(rows_affected, num_bookmarks_to_delete as u64);

        let mut bookmarks = get_all_bookmarks(&fx.pool)
            .await
            .expect("bookmarks should've been fetched");
        bookmarks.sort_by(|a, b| a.uri.cmp(&b.uri));
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://retained-one.com"
          title: retained bookmark
          tags: "retained-only,shared"
        - uri: "https://retained-two.com"
          title: ~
          tags: ~
        "#);

        let tags_in_db = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags_in_db, @r"
        - retained-only
        - shared
        ");
    }

    #[tokio::test]
    async fn deleting_tags_works() {
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

        for (uri, title, tags) in uris {
            let draft_bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmark should be initialized");
            create_or_update_bookmark(
                &fx.pool,
                &draft_bookmark,
                now,
                SaveBookmarkOptions::default(),
            )
            .await
            .expect("bookmark should be saved in db");
        }

        // WHEN
        let tags_to_delete = ["tag1", "tag2", "absent-tag"]
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>();
        let num_rows_deleted = delete_tags_by_name(&fx.pool, &tags_to_delete)
            .await
            .expect("result should've been a success");

        // THEN
        assert_eq!(num_rows_deleted, 2);

        let tags_left = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");
        assert_yaml_snapshot!(tags_left, @r"
        - tag3
        - tag4
        - tag5
        ");
    }
}
