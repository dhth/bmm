use super::DBError;
use crate::domain::{SavedBookmark, TagStats};
use sqlx::{Pool, Sqlite};

const SEARCH_TERMS_UPPER_LIMIT: usize = 10;

#[derive(thiserror::Error, Debug)]
pub enum SearchTermsError {
    #[error("query is empty")]
    QueryEmpty,
    #[error("too many terms (maximum allowed: {SEARCH_TERMS_UPPER_LIMIT})")]
    TooManyTerms,
}

#[derive(Debug, Clone)]
pub struct SearchTerms(Vec<String>);

impl SearchTerms {
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.0.iter()
    }
}

impl TryFrom<&str> for SearchTerms {
    type Error = SearchTermsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(SearchTermsError::QueryEmpty);
        }

        let mut terms = value
            .trim()
            .split(" ")
            .filter(|t| !t.trim().is_empty())
            .collect::<Vec<_>>();

        terms.sort();
        terms.dedup();

        if terms.len() > SEARCH_TERMS_UPPER_LIMIT {
            return Err(SearchTermsError::TooManyTerms);
        }

        Ok(Self(
            terms.into_iter().map(|t| t.to_string()).collect::<Vec<_>>(),
        ))
    }
}

impl TryFrom<&Vec<String>> for SearchTerms {
    type Error = SearchTermsError;

    fn try_from(value: &Vec<String>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(SearchTermsError::QueryEmpty);
        }

        let mut terms = value
            .iter()
            .filter(|t| !t.trim().is_empty())
            .collect::<Vec<_>>();

        terms.sort();
        terms.dedup();

        if terms.len() > SEARCH_TERMS_UPPER_LIMIT {
            return Err(SearchTermsError::TooManyTerms);
        }

        Ok(Self(
            terms.into_iter().map(|t| t.to_string()).collect::<Vec<_>>(),
        ))
    }
}

pub async fn get_bookmark_with_exact_uri(
    pool: &Pool<Sqlite>,
    uri: &str,
) -> Result<Option<SavedBookmark>, DBError> {
    let maybe_bookmark = sqlx::query_as!(
        SavedBookmark,
        r#"
SELECT
    uri,
    title,
    (
        SELECT
            GROUP_CONCAT(t.name, ',' ORDER BY t.name ASC)
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS "tags: String"
FROM
    bookmarks b
WHERE
    uri = ?
"#,
        uri
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DBError::CouldntExecuteQuery("fetch bookmark by exact uri".into(), e))?;

    Ok(maybe_bookmark)
}

pub async fn get_bookmarks(
    pool: &Pool<Sqlite>,
    uri_query: Option<String>,
    title_query: Option<String>,
    tags_query: Vec<String>,
    limit: u16,
) -> Result<Vec<SavedBookmark>, DBError> {
    match (uri_query, title_query, tags_query.as_slice()) {
        (None, None, []) => sqlx::query_as!(
            SavedBookmark,
            r#"
SELECT
    uri,
    title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
ORDER BY
    updated_at DESC
LIMIT
    ?
"#,
            limit
        )
        .fetch_all(pool)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("query bookmarks".into(), e)),
        (Some(u), None, []) => {
            let uri_query = format!("%{u}%");

            sqlx::query_as!(
                SavedBookmark,
                r#"
SELECT
    uri,
    title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
WHERE
    b.uri LIKE ?
ORDER BY
    b.updated_at DESC
LIMIT
    ?
"#,
                uri_query,
                limit
            )
            .fetch_all(pool)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("query bookmarks by uri".into(), e))
        }
        (None, Some(d), []) => {
            let title_query = format!("%{d}%");

            sqlx::query_as!(
                SavedBookmark,
                r#"
SELECT
    uri,
    title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
WHERE
    title LIKE ?
ORDER BY
    updated_at DESC
LIMIT
    ?
"#,
                title_query,
                limit
            )
            .fetch_all(pool)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("query bookmarks by title".into(), e))
        }
        (None, None, tags) => {
            let query = format!(
                r#"
SELECT
    b.uri,
    b.title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
    JOIN bookmark_tags bt ON b.id = bt.bookmark_id
    JOIN tags t ON bt.tag_id = t.id
WHERE
    t.name IN ({})
GROUP BY
    b.id,
    b.uri,
    b.title,
    b.updated_at
HAVING
    COUNT(DISTINCT t.name) = ?
ORDER BY
    updated_at DESC
LIMIT
    ?
"#,
                tags.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
            );
            let mut query_builder = sqlx::query_as::<_, SavedBookmark>(&query);
            for tag in tags {
                query_builder = query_builder.bind(tag);
            }

            query_builder = query_builder.bind(tags.len() as i64);
            query_builder = query_builder.bind(limit);

            let bookmarks = query_builder
                .fetch_all(pool)
                .await
                .map_err(|e| DBError::CouldntExecuteQuery("query bookmarks by tags".into(), e))?;

            Ok(bookmarks)
        }
        (Some(u), Some(d), []) => {
            let uri_query = format!("%{u}%");
            let title_query = format!("%{d}%");

            sqlx::query_as!(
                SavedBookmark,
                r#"
SELECT
    uri,
    title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
WHERE
    uri LIKE ?
    AND title LIKE ?
ORDER BY
    updated_at DESC
LIMIT
    ?
"#,
                uri_query,
                title_query,
                limit
            )
            .fetch_all(pool)
            .await
            .map_err(|e| DBError::CouldntExecuteQuery("query bookmarks by uri and title".into(), e))
        }
        (Some(u), None, tags) => {
            let query = format!(
                r#"
SELECT
    b.uri,
    b.title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
    JOIN bookmark_tags bt ON b.id = bt.bookmark_id
    JOIN tags t ON bt.tag_id = t.id
WHERE
    b.uri LIKE ?
    AND t.name IN ({})
GROUP BY
    b.id,
    b.uri,
    b.title,
    b.updated_at
HAVING
    COUNT(DISTINCT t.name) = ?
ORDER BY
    b.updated_at DESC
LIMIT
    ?
"#,
                tags.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
            );
            let mut query_builder = sqlx::query_as::<_, SavedBookmark>(&query);
            query_builder = query_builder.bind(format!("%{u}%"));
            for tag in tags {
                query_builder = query_builder.bind(tag);
            }

            query_builder = query_builder.bind(tags.len() as i64);
            query_builder = query_builder.bind(limit);

            let bookmarks = query_builder.fetch_all(pool).await.map_err(|e| {
                DBError::CouldntExecuteQuery("query bookmarks by uri and tags".into(), e)
            })?;

            Ok(bookmarks)
        }
        (None, Some(d), tags) => {
            let query = format!(
                r#"
SELECT
    b.uri,
    b.title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
    JOIN bookmark_tags bt ON b.id = bt.bookmark_id
    JOIN tags t ON bt.tag_id = t.id
WHERE
    b.title LIKE ?
    AND t.name IN ({})
GROUP BY
    b.id,
    b.uri,
    b.title,
    b.updated_at
HAVING
    COUNT(DISTINCT t.name) = ?
ORDER BY
    b.updated_at DESC
LIMIT
    ?
"#,
                tags.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
            );
            let mut query_builder = sqlx::query_as::<_, SavedBookmark>(&query);
            query_builder = query_builder.bind(format!("%{d}%"));
            for tag in tags {
                query_builder = query_builder.bind(tag);
            }

            query_builder = query_builder.bind(tags.len() as i64);
            query_builder = query_builder.bind(limit);

            let bookmarks = query_builder.fetch_all(pool).await.map_err(|e| {
                DBError::CouldntExecuteQuery("query bookmarks by title and tags".into(), e)
            })?;

            Ok(bookmarks)
        }
        (Some(u), Some(d), tags) => {
            let query = format!(
                r#"
SELECT
    b.uri,
    b.title,
    (
        SELECT
            GROUP_CONCAT(
                t.name,
                ','
                ORDER BY
                    t.name ASC
            )
        FROM
            tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
        WHERE
            bt.bookmark_id = b.id
    ) AS tags
FROM
    bookmarks b
    JOIN bookmark_tags bt ON b.id = bt.bookmark_id
    JOIN tags t ON bt.tag_id = t.id
WHERE
    b.uri LIKE ?
    AND b.title LIKE ?
    AND t.name IN ({})
GROUP BY
    b.id,
    b.uri,
    b.title,
    b.updated_at
HAVING
    COUNT(DISTINCT t.name) = ?
ORDER BY
    b.updated_at DESC
LIMIT
    ?
"#,
                tags.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
            );
            let mut query_builder = sqlx::query_as::<_, SavedBookmark>(&query);
            query_builder = query_builder.bind(format!("%{u}%"));
            query_builder = query_builder.bind(format!("%{d}%"));
            for tag in tags {
                query_builder = query_builder.bind(tag);
            }

            query_builder = query_builder.bind(tags.len() as i64);
            query_builder = query_builder.bind(limit);

            let bookmarks = query_builder.fetch_all(pool).await.map_err(|e| {
                DBError::CouldntExecuteQuery("query bookmarks by uri and tags".into(), e)
            })?;

            Ok(bookmarks)
        }
    }
}

pub async fn get_bookmarks_by_query(
    pool: &Pool<Sqlite>,
    search_terms: &SearchTerms,
    limit: u16,
) -> Result<Vec<SavedBookmark>, DBError> {
    let query = format!(
        r#"
SELECT
    b.uri,
    b.title,
    GROUP_CONCAT(
        t.name,
        ','
        ORDER BY
            t.name ASC
    ) AS tags
FROM
    bookmarks b
    LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
    LEFT JOIN tags t ON bt.tag_id = t.id
WHERE
    {}
GROUP BY
    b.id,
    b.uri,
    b.title,
    b.updated_at
ORDER BY
    b.updated_at DESC
LIMIT
    ?
"#,
        search_terms
            .iter()
        .map(|_| "(b.uri LIKE ? OR b.title LIKE ? OR EXISTS (SELECT 1 FROM tags t JOIN bookmark_tags bt ON t.id = bt.tag_id WHERE bt.bookmark_id = b.id AND t.name LIKE ?))")
            .collect::<Vec<&str>>()
            .join(" AND ")
    );

    let mut query_builder = sqlx::query_as::<_, SavedBookmark>(&query);

    let search_terms_with_like_markers = search_terms
        .iter()
        .map(|t| format!("%{t}%"))
        .collect::<Vec<_>>();

    for term in search_terms_with_like_markers.iter() {
        query_builder = query_builder.bind(term);
        query_builder = query_builder.bind(term);
        query_builder = query_builder.bind(term);
    }

    query_builder = query_builder.bind(limit);

    query_builder.fetch_all(pool).await.map_err(|e| {
        DBError::CouldntExecuteQuery("get bookmarks where any attribute matches query".into(), e)
    })
}

#[allow(unused)]
pub(super) async fn get_num_bookmarks(pool: &Pool<Sqlite>) -> Result<i64, DBError> {
    sqlx::query_scalar!(
        "
SELECT
    COUNT(*)
FROM
    bookmarks
",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| DBError::CouldntExecuteQuery("fetch number of bookmarks".into(), e))
}

#[allow(unused)]
pub async fn does_tag_exist(pool: &Pool<Sqlite>, tag: &str) -> Result<Option<i64>, DBError> {
    let maybe_id = sqlx::query!(
        "
SELECT
    id
FROM
    tags
WHERE
    name = ?
",
        tag
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DBError::CouldntExecuteQuery("check if tag exists".into(), e))?
    .map(|r| r.id);

    Ok(maybe_id)
}

pub async fn get_tags(pool: &Pool<Sqlite>) -> Result<Vec<String>, DBError> {
    let tag_names = sqlx::query!(
        "
SELECT
    t.name
FROM
    tags t
ORDER BY name
"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DBError::CouldntExecuteQuery("fetch tags with stats".into(), e))?
    .into_iter()
    .map(|r| r.name)
    .collect();

    Ok(tag_names)
}

pub async fn get_tags_with_stats(pool: &Pool<Sqlite>) -> Result<Vec<TagStats>, DBError> {
    let tag_names = sqlx::query_as!(
        TagStats,
        "
SELECT
    t.name, count(bt.bookmark_id) as num_bookmarks
FROM
    tags t
    LEFT JOIN bookmark_tags bt ON bt.tag_id = t.id
GROUP BY
	t.id
ORDER BY name
"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DBError::CouldntExecuteQuery("fetch tags with stats".into(), e))?;

    Ok(tag_names)
}

#[cfg(test)]
mod tests {
    use super::super::create_or_update_bookmark;
    use super::super::test_fixtures::DBPoolFixture;
    use super::*;
    use crate::{
        domain::{DraftBookmark, PotentialBookmark},
        persistence::SaveBookmarkOptions,
    };

    use insta::assert_yaml_snapshot;
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn save_test_bookmarks(pool: &Pool<Sqlite>) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        let uris = [
            (
                "https://github.com/launchbadge/sqlx",
                Some("sqlx's github page"),
                vec!["sql", "sqlite", "github", "crate"],
                now,
            ),
            (
                "https://github.com/serde-rs/serde",
                None,
                vec!["serde", "github"],
                now - 100,
            ),
            (
                "https://github.com/clap-rs/clap",
                Some("clap repository on github"),
                vec!["cli", "clap"],
                now - 200,
            ),
            (
                "https://crates.io/crates/anyhow",
                Some("anyhow on crates.io PaGe"),
                vec!["github", "error-handling", "crate"],
                now - 300,
            ),
            (
                "https://crates.io/crates/thiserror",
                None,
                vec!["error-handling", "github", "crate"],
                now - 400,
            ),
        ];

        for (uri, title, tags, now) in uris {
            let draft_bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmark should've been initialized");

            create_or_update_bookmark(pool, &draft_bookmark, now, SaveBookmarkOptions::default())
                .await
                .expect("bookmark should be saved in db");
        }
    }

    #[tokio::test]
    async fn get_bookmark_with_uri_returns_none_if_bookmark_doesnt_exist() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;

        // WHEN
        let maybe_bookmark = get_bookmark_with_exact_uri(&fixture.pool, "https://blah.com")
            .await
            .unwrap();

        // THEN
        assert!(maybe_bookmark.is_none())
    }

    #[tokio::test]
    async fn get_bookmark_with_uri_returns_bookmark_when_present() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let title = Some("sqlx's github page");
        let tags = vec!["sql", "crate", "github"];
        let draft_bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
            .expect("draft bookmark should be initialized");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        create_or_update_bookmark(
            &fixture.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should be saved in db");

        // WHEN
        let bookmark = get_bookmark_with_exact_uri(&fixture.pool, uri)
            .await
            .unwrap()
            .expect("result should've been a bookmark");

        // THEN
        assert_yaml_snapshot!(bookmark, @r#"
        uri: "https://github.com/launchbadge/sqlx"
        title: "sqlx's github page"
        tags: "crate,github,sql"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_uri_only_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let uri_query = Some("github.com".into());
        let bookmarks = get_bookmarks(&fixture.pool, uri_query, None, Vec::new(), 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/launchbadge/sqlx"
          title: "sqlx's github page"
          tags: "crate,github,sql,sqlite"
        - uri: "https://github.com/serde-rs/serde"
          title: ~
          tags: "github,serde"
        - uri: "https://github.com/clap-rs/clap"
          title: clap repository on github
          tags: "clap,cli"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_title_only_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let title_query = Some("page".into());
        let bookmarks = get_bookmarks(&fixture.pool, None, title_query, Vec::new(), 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/launchbadge/sqlx"
          title: "sqlx's github page"
          tags: "crate,github,sql,sqlite"
        - uri: "https://crates.io/crates/anyhow"
          title: anyhow on crates.io PaGe
          tags: "crate,error-handling,github"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_a_tag_only_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let tags_query = vec!["serde".into()];
        let bookmarks = get_bookmarks(&fixture.pool, None, None, tags_query, 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/serde-rs/serde"
          title: ~
          tags: "github,serde"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_multiple_tags_only_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let tags_query = vec!["github".into(), "crate".into()];
        let bookmarks = get_bookmarks(&fixture.pool, None, None, tags_query, 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/launchbadge/sqlx"
          title: "sqlx's github page"
          tags: "crate,github,sql,sqlite"
        - uri: "https://crates.io/crates/anyhow"
          title: anyhow on crates.io PaGe
          tags: "crate,error-handling,github"
        - uri: "https://crates.io/crates/thiserror"
          title: ~
          tags: "crate,error-handling,github"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_both_uri_and_tags_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let uri_query = Some("crate".into());
        let tags_query = vec!["github".into()];
        let bookmarks = get_bookmarks(&fixture.pool, uri_query, None, tags_query, 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://crates.io/crates/anyhow"
          title: anyhow on crates.io PaGe
          tags: "crate,error-handling,github"
        - uri: "https://crates.io/crates/thiserror"
          title: ~
          tags: "crate,error-handling,github"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_both_uri_and_title_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let uri_query = Some("github".into());
        let title_query = Some("repository".into());
        let bookmarks = get_bookmarks(&fixture.pool, uri_query, title_query, Vec::new(), 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/clap-rs/clap"
          title: clap repository on github
          tags: "clap,cli"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_both_title_and_tags_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let title_query = Some("crate".into());
        let tags_query = vec!["error-handling".into()];
        let bookmarks = get_bookmarks(&fixture.pool, None, title_query, tags_query, 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://crates.io/crates/anyhow"
          title: anyhow on crates.io PaGe
          tags: "crate,error-handling,github"
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_all_three_attributes_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let uri_query = Some("github".into());
        let title_query = Some("page".into());
        let tags_query = vec!["sql".into(), "crate".into()];
        let bookmarks = get_bookmarks(&fixture.pool, uri_query, title_query, tags_query, 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/launchbadge/sqlx"
          title: "sqlx's github page"
          tags: "crate,github,sql,sqlite"
        "#);
    }

    #[tokio::test]
    async fn limiting_search_results_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let uri_query = Some("github.com".into());
        let bookmarks = get_bookmarks(&fixture.pool, uri_query, None, Vec::new(), 2)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/launchbadge/sqlx"
          title: "sqlx's github page"
          tags: "crate,github,sql,sqlite"
        - uri: "https://github.com/serde-rs/serde"
          title: ~
          tags: "github,serde"
        "#);
    }

    #[tokio::test]
    async fn getting_returns_results_in_order_of_last_update() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        create_or_update_bookmark(
            &fixture.pool,
            &DraftBookmark::try_from(PotentialBookmark::from((
                "https://github.com/launchbadge/sqlx",
                None,
                None,
            )))
            .expect("draft bookmark 1 should be initialized"),
            now - 200 * 60,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark 1 should be saved in db");

        create_or_update_bookmark(
            &fixture.pool,
            &DraftBookmark::try_from(PotentialBookmark::from((
                "https://github.com/serde-rs/serde",
                None,
                None,
            )))
            .expect("draft bookmark 2 should be initialized"),
            now - 150 * 60,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark 2 should be saved in db");

        create_or_update_bookmark(
            &fixture.pool,
            &DraftBookmark::try_from(PotentialBookmark::from((
                "https://github.com/clap-rs/clap",
                None,
                None,
            )))
            .expect("draft bookmark 3 should be initialized"),
            now - 100 * 60,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark 3 should be saved in db");

        create_or_update_bookmark(
            &fixture.pool,
            &DraftBookmark::try_from(PotentialBookmark::from((
                "https://crates.io/crates/anyhow",
                None,
                None,
            )))
            .expect("draft bookmark 4 should be initialized"),
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark 4 should be saved in db");

        // WHEN
        let uri_query = Some("github.com".into());
        let bookmarks = get_bookmarks(&fixture.pool, uri_query, None, Vec::new(), 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/clap-rs/clap"
          title: ~
          tags: ~
        - uri: "https://github.com/serde-rs/serde"
          title: ~
          tags: ~
        - uri: "https://github.com/launchbadge/sqlx"
          title: ~
          tags: ~
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_query_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        let uris = [
            (
                "https://uri-one-keyword1-keyword3.com",
                Some("title one keyword2 keyword4"),
                vec!["prefix1-tag", "prefix2-tag", "tag-suffix1"],
            ),
            (
                "https://uri-two.com",
                None,
                vec![
                    "prefix2-tag",
                    "tag-suffix2",
                    "tag-keyword2",
                    "tag-keyword3",
                    "tag-keyword4",
                ],
            ),
            (
                "https://three-keyword4.com",
                Some("title three keyword1"),
                vec![],
            ),
        ];

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        for (uri, title, tags) in uris {
            let draft_bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
                    .expect("draft bookmark should be initialized");
            create_or_update_bookmark(
                &fixture.pool,
                &draft_bookmark,
                now,
                SaveBookmarkOptions::default(),
            )
            .await
            .expect("bookmark should be saved in db");
        }

        let test_cases: Vec<(SearchTerms, usize)> = vec![
            (SearchTerms::try_from("absent").unwrap(), 0),   // none
            (SearchTerms::try_from("uri").unwrap(), 2),      // uri only
            (SearchTerms::try_from("title").unwrap(), 2),    // title only
            (SearchTerms::try_from("prefix2").unwrap(), 2),  // tags only
            (SearchTerms::try_from("keyword1").unwrap(), 2), // uri + title
            (SearchTerms::try_from("keyword2").unwrap(), 2), // title + tags
            (SearchTerms::try_from("keyword3").unwrap(), 2), // uri + tags
            (SearchTerms::try_from("keyword4").unwrap(), 3), // uri + title + tags
            (
                SearchTerms::try_from("https keyword one prefix2-tag tag-suffix1").unwrap(),
                1,
            ), // multiple terms
            (SearchTerms::try_from("uri prefix2 keyword3").unwrap(), 2), // multiple terms
            (SearchTerms::try_from("three keyword").unwrap(), 1), // multiple terms
            (
                SearchTerms::try_from("title prefix2 uri one tag-suffix1").unwrap(),
                1,
            ), // multiple terms
        ];

        // WHEN
        for (query, expected_num_bookmarks) in test_cases {
            let bookmarks = get_bookmarks_by_query(&fixture.pool, &query, 10)
                .await
                .unwrap();

            // THEN
            assert_eq!(
                bookmarks.len(),
                expected_num_bookmarks,
                "failed for query: {:?}",
                &query
            );
        }
    }

    #[tokio::test]
    async fn searching_bookmarks_by_query_returns_all_data_for_each_bookmark() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
        save_test_bookmarks(&fixture.pool).await;

        // WHEN
        let search_terms = SearchTerms::try_from("crate page").unwrap();
        let bookmarks = get_bookmarks_by_query(&fixture.pool, &search_terms, 10)
            .await
            .unwrap();

        // THEN
        assert_yaml_snapshot!(bookmarks, @r#"
        - uri: "https://github.com/launchbadge/sqlx"
          title: "sqlx's github page"
          tags: "crate,github,sql,sqlite"
        - uri: "https://crates.io/crates/anyhow"
          title: anyhow on crates.io PaGe
          tags: "crate,error-handling,github"
        "#);
    }

    #[tokio::test]
    async fn getting_tags_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
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
                &fixture.pool,
                &draft_bookmark,
                now,
                SaveBookmarkOptions::default(),
            )
            .await
            .expect("bookmark should be saved in db");
        }

        // WHEN
        let tags = get_tags(&fixture.pool)
            .await
            .expect("tags should've been fetched");

        // THEN
        assert_eq!(tags.len(), 5);
        assert_eq!(
            tags.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            vec!["tag1", "tag2", "tag3", "tag4", "tag5",]
        );
    }

    #[tokio::test]
    async fn getting_tags_with_stats_works() {
        // GIVEN
        let fixture = DBPoolFixture::new().await;
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
                &fixture.pool,
                &draft_bookmark,
                now,
                SaveBookmarkOptions::default(),
            )
            .await
            .expect("bookmark should be saved in db");
        }

        // WHEN
        let tags = get_tags_with_stats(&fixture.pool)
            .await
            .expect("tags should've been fetched");

        // THEN
        assert_eq!(tags.len(), 5);
        assert_eq!(
            tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            vec!["tag1", "tag2", "tag3", "tag4", "tag5",]
        );
        assert_eq!(
            tags.iter().map(|t| t.num_bookmarks).collect::<Vec<_>>(),
            vec![1, 3, 4, 1, 1]
        );
    }
}
