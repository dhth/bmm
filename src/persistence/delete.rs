use super::DBError;
use sqlx::{Pool, Sqlite};

pub async fn delete_bookmarks_with_uris(
    pool: &Pool<Sqlite>,
    uris: &Vec<String>,
) -> Result<u64, DBError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(DBError::CouldntBeginTransaction)?;

    let rows_affected = {
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
            uris.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
        );

        let mut query_builder = sqlx::query::<_>(&query);
        for uri in uris {
            query_builder = query_builder.bind(uri.as_str());
        }

        let result = query_builder
            .execute(&mut *tx)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("delete bookmarks with uris".into(), e))?;

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

        result.rows_affected()
    };

    tx.commit()
        .await
        .map_err(DBError::CouldntCommitTransaction)?;

    Ok(rows_affected)
}

#[cfg(test)]
mod tests {
    use super::super::test_fixtures::DBPoolFixture;
    use super::super::{create_or_update_bookmark, get_num_bookmarks, get_tags};
    use super::*;
    use crate::domain::DraftBookmark;
    use pretty_assertions::assert_eq;
    use std::time::{SystemTime, UNIX_EPOCH};

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[tokio::test]
    async fn deleting_uris_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let num_bookmarks = 10;

        for i in 1..=num_bookmarks {
            let uri = format!("https://uri-{}.com", i);
            let draft_bookmark = DraftBookmark::try_from((uri.as_str(), None, vec![]))
                .expect("draft bookmark should've been created");
            println!("draft_bookmark: {}", draft_bookmark.uri());
            create_or_update_bookmark(&fixture.pool, &draft_bookmark, now)
                .await
                .expect("bookmark should've been saved in db");
        }

        // WHEN
        let uris_to_delete = vec!["https://uri-1.com".into(), "https://uri-4.com".into()];

        let result = delete_bookmarks_with_uris(&fixture.pool, &uris_to_delete)
            .await
            .expect("result should've been a success");
        assert_eq!(result, uris_to_delete.len() as u64);

        let num_bookmarks_in_db = get_num_bookmarks(&fixture.pool)
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
        let fixture = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let uri = "https://uri.com";
        let draft_bookmark = DraftBookmark::try_from((uri, None, vec![]))
            .expect("draft bookmark should've been created");
        println!("draft_bookmark: {}", draft_bookmark.uri());
        create_or_update_bookmark(&fixture.pool, &draft_bookmark, now)
            .await
            .expect("bookmark should've been saved in db");

        // WHEN
        let uris_to_delete = vec![
            "https://unknown-uri-1.com".into(),
            "https://unknown-uri-2.com".into(),
        ];

        let result = delete_bookmarks_with_uris(&fixture.pool, &uris_to_delete)
            .await
            .expect("result should've been a success");
        assert_eq!(result, 0);

        let num_bookmarks_in_db = get_num_bookmarks(&fixture.pool)
            .await
            .expect("number of bookmarks should've been fetched");
        assert_eq!(num_bookmarks_in_db, 1);
    }

    #[tokio::test]
    async fn deleting_uris_cleans_up_unused_tags() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        let uri = "https://uri.com";
        let draft_bookmark = DraftBookmark::try_from((uri, None, vec!["tag"]))
            .expect("draft bookmark should've been created");
        println!("draft_bookmark: {}", draft_bookmark.uri());
        create_or_update_bookmark(&fixture.pool, &draft_bookmark, now)
            .await
            .expect("bookmark should've been saved in db");

        // WHEN
        let uris_to_delete = vec![uri.to_string()];

        let result = delete_bookmarks_with_uris(&fixture.pool, &uris_to_delete)
            .await
            .expect("result should've been a success");
        assert_eq!(result, 1);

        let tags_in_db = get_tags(&fixture.pool)
            .await
            .expect("tags should've been fetched");
        assert!(tags_in_db.is_empty());
    }
}
