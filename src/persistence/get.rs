use super::DBError;
use crate::domain::{SavedBookmark, TagStats};
use sqlx::{Pool, QueryBuilder, Sqlite};

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
    pub fn iter(&self) -> std::slice::Iter<'_, String> {
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

#[derive(Clone, Copy, Debug)]
pub enum UriMatchMode {
    Exact,
    Pattern,
}

pub async fn get_matching_bookmark_uris(
    pool: &Pool<Sqlite>,
    values: &[String],
    mode: UriMatchMode,
) -> Result<Vec<String>, DBError> {
    if values.is_empty() {
        return Ok(Vec::new());
    }

    let mut query_builder = QueryBuilder::<Sqlite>::new(
        r#"
SELECT
    uri
FROM
    bookmarks
WHERE
    "#,
    );

    match mode {
        UriMatchMode::Exact => {
            query_builder.push("uri IN (");
            let mut values_builder = query_builder.separated(", ");
            for value in values {
                values_builder.push_bind(value);
            }
            values_builder.push_unseparated(")");
        }
        UriMatchMode::Pattern => {
            let mut patterns_builder = query_builder.separated(" OR ");
            for value in values {
                patterns_builder
                    .push("uri LIKE ")
                    .push_bind_unseparated(literal_like_substring_pattern(value))
                    .push_unseparated(" ESCAPE '\\'");
            }
        }
    }

    query_builder.push(" ORDER BY uri");

    query_builder
        .build_query_scalar::<String>()
        .fetch_all(pool)
        .await
        .map_err(|e| DBError::CouldntExecuteQuery("fetch matching bookmark uris".into(), e))
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
            let uri_query = literal_like_substring_pattern(&u);

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
    b.uri LIKE ? ESCAPE '\'
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
            let title_query = literal_like_substring_pattern(&d);

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
    title LIKE ? ESCAPE '\'
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
            let uri_query = literal_like_substring_pattern(&u);
            let title_query = literal_like_substring_pattern(&d);

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
    uri LIKE ? ESCAPE '\'
    AND title LIKE ? ESCAPE '\'
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
    b.uri LIKE ? ESCAPE '\'
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
            query_builder = query_builder.bind(literal_like_substring_pattern(&u));
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
    b.title LIKE ? ESCAPE '\'
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
            query_builder = query_builder.bind(literal_like_substring_pattern(&d));
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
    b.uri LIKE ? ESCAPE '\'
    AND b.title LIKE ? ESCAPE '\'
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
            query_builder = query_builder.bind(literal_like_substring_pattern(&u));
            query_builder = query_builder.bind(literal_like_substring_pattern(&d));
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
        .map(|_| "(b.uri LIKE ? ESCAPE '\\' OR b.title LIKE ? ESCAPE '\\' OR EXISTS (SELECT 1 FROM tags t JOIN bookmark_tags bt ON t.id = bt.tag_id WHERE bt.bookmark_id = b.id AND t.name LIKE ? ESCAPE '\\'))")
            .collect::<Vec<&str>>()
            .join(" AND ")
    );

    let mut query_builder = sqlx::query_as::<_, SavedBookmark>(&query);

    let search_terms_with_like_markers = search_terms
        .iter()
        .map(|t| literal_like_substring_pattern(t))
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
pub async fn get_all_bookmarks(pool: &Pool<Sqlite>) -> Result<Vec<SavedBookmark>, DBError> {
    get_bookmarks(pool, None, None, vec![], 1000).await
}

fn literal_like_substring_pattern(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for character in value.chars() {
        if matches!(character, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
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
    async fn getting_matching_bookmark_uris_exactly_returns_existing_values() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;
        let values = vec![
            "https://github.com/serde-rs/serde".to_string(),
            "https://does-not-exist.com".to_string(),
            "https://github.com/launchbadge/sqlx".to_string(),
            "https://github.com/serde-rs/serde".to_string(),
        ];

        // WHEN
        let uris = get_matching_bookmark_uris(&fx.pool, &values, UriMatchMode::Exact)
            .await
            .expect("matching bookmark uris should've been fetched");

        // THEN
        assert_yaml_snapshot!(uris, @r#"
        - "https://github.com/launchbadge/sqlx"
        - "https://github.com/serde-rs/serde"
        "#);
    }

    #[tokio::test]
    async fn getting_matching_bookmark_uris_by_pattern_uses_union_semantics() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;
        let values = vec!["github.com".to_string(), "serde".to_string()];

        // WHEN
        let uris = get_matching_bookmark_uris(&fx.pool, &values, UriMatchMode::Pattern)
            .await
            .expect("matching bookmark uris should've been fetched");

        // THEN
        assert_yaml_snapshot!(uris, @r#"
        - "https://github.com/clap-rs/clap"
        - "https://github.com/launchbadge/sqlx"
        - "https://github.com/serde-rs/serde"
        "#);
    }

    #[tokio::test]
    async fn get_matching_bookmark_uris_treats_like_metacharacters_literally() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        for uri in [
            "https://example.com/under_score",
            "https://example.com/underXscore",
            "https://example.com/percent%20value",
            "https://example.com/percentZZ20value",
        ] {
            let bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, None, None)))
                .expect("draft bookmark should've been initialized");
            create_or_update_bookmark(&fx.pool, &bookmark, 0, SaveBookmarkOptions::default())
                .await
                .expect("bookmark should be saved in db");
        }

        // WHEN
        let values = vec!["under_score".to_string(), "%20".to_string()];
        let uris = get_matching_bookmark_uris(&fx.pool, &values, UriMatchMode::Pattern)
            .await
            .expect("matching bookmark uris should've been fetched");

        // THEN
        assert_yaml_snapshot!(uris, @r#"
        - "https://example.com/percent%20value"
        - "https://example.com/under_score"
        "#);
    }

    #[tokio::test]
    async fn getting_matching_bookmark_uris_returns_nothing_when_no_values_match() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;
        let values = vec!["does-not-exist".to_string()];

        // WHEN
        let exact_uris = get_matching_bookmark_uris(&fx.pool, &values, UriMatchMode::Exact)
            .await
            .expect("matching bookmark uris should've been fetched");
        let pattern_uris = get_matching_bookmark_uris(&fx.pool, &values, UriMatchMode::Pattern)
            .await
            .expect("matching bookmark uris should've been fetched");

        // THEN
        assert!(exact_uris.is_empty());
        assert!(pattern_uris.is_empty());
    }

    #[tokio::test]
    async fn getting_matching_bookmark_uris_returns_nothing_for_empty_values() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let exact_uris = get_matching_bookmark_uris(&fx.pool, &[], UriMatchMode::Exact)
            .await
            .expect("matching bookmark uris should've been fetched");
        let pattern_uris = get_matching_bookmark_uris(&fx.pool, &[], UriMatchMode::Pattern)
            .await
            .expect("matching bookmark uris should've been fetched");

        // THEN
        assert!(exact_uris.is_empty());
        assert!(pattern_uris.is_empty());
    }

    #[tokio::test]
    async fn get_bookmark_with_uri_returns_none_if_bookmark_doesnt_exist() {
        // GIVEN
        let fx = DBPoolFixture::new().await;

        // WHEN
        let maybe_bookmark = get_bookmark_with_exact_uri(&fx.pool, "https://blah.com")
            .await
            .unwrap();

        // THEN
        assert!(maybe_bookmark.is_none())
    }

    #[tokio::test]
    async fn get_bookmark_with_uri_returns_bookmark_when_present() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        let uri = "https://github.com/launchbadge/sqlx";
        let title = Some("sqlx's github page");
        let tags = vec!["sql", "crate", "github"];
        let draft_bookmark = DraftBookmark::try_from(PotentialBookmark::from((uri, title, &tags)))
            .expect("draft bookmark should be initialized");
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;
        create_or_update_bookmark(
            &fx.pool,
            &draft_bookmark,
            now,
            SaveBookmarkOptions::default(),
        )
        .await
        .expect("bookmark should be saved in db");

        // WHEN
        let bookmark = get_bookmark_with_exact_uri(&fx.pool, uri)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let uri_query = Some("github.com".into());
        let bookmarks = get_bookmarks(&fx.pool, uri_query, None, Vec::new(), 10)
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
    async fn bookmark_filters_treat_like_metacharacters_literally() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        for (uri, title) in [
            (
                "https://example.com/under_score",
                r"100% literal back\slash",
            ),
            (
                "https://example.com/underXscore",
                "100 percent literal backXslash",
            ),
            ("https://example.com/percent%20value", "encoded value"),
            ("https://example.com/percentZZ20value", "plain value"),
        ] {
            let bookmark =
                DraftBookmark::try_from(PotentialBookmark::from((uri, Some(title), None)))
                    .expect("draft bookmark should've been initialized");
            create_or_update_bookmark(&fx.pool, &bookmark, 0, SaveBookmarkOptions::default())
                .await
                .expect("bookmark should be saved in db");
        }

        // WHEN
        let by_uri = get_bookmarks(&fx.pool, Some("under_score".into()), None, vec![], 10)
            .await
            .expect("bookmarks should've been fetched");
        let by_title = get_bookmarks(&fx.pool, None, Some("100%".into()), vec![], 10)
            .await
            .expect("bookmarks should've been fetched");
        let by_escape_character =
            get_bookmarks(&fx.pool, None, Some(r"back\slash".into()), vec![], 10)
                .await
                .expect("bookmarks should've been fetched");
        let search_terms = SearchTerms::try_from("%20").expect("search terms should be valid");
        let by_search = get_bookmarks_by_query(&fx.pool, &search_terms, 10)
            .await
            .expect("bookmarks should've been fetched");

        // THEN
        assert_yaml_snapshot!(by_uri, @r#"
        - uri: "https://example.com/under_score"
          title: "100% literal back\\slash"
          tags: ~
        "#);
        assert_yaml_snapshot!(by_title, @r#"
        - uri: "https://example.com/under_score"
          title: "100% literal back\\slash"
          tags: ~
        "#);
        assert_yaml_snapshot!(by_escape_character, @r#"
        - uri: "https://example.com/under_score"
          title: "100% literal back\\slash"
          tags: ~
        "#);
        assert_yaml_snapshot!(by_search, @r#"
        - uri: "https://example.com/percent%20value"
          title: encoded value
          tags: ~
        "#);
    }

    #[tokio::test]
    async fn getting_bookmarks_by_title_only_works() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let title_query = Some("page".into());
        let bookmarks = get_bookmarks(&fx.pool, None, title_query, Vec::new(), 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let tags_query = vec!["serde".into()];
        let bookmarks = get_bookmarks(&fx.pool, None, None, tags_query, 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let tags_query = vec!["github".into(), "crate".into()];
        let bookmarks = get_bookmarks(&fx.pool, None, None, tags_query, 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let uri_query = Some("crate".into());
        let tags_query = vec!["github".into()];
        let bookmarks = get_bookmarks(&fx.pool, uri_query, None, tags_query, 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let uri_query = Some("github".into());
        let title_query = Some("repository".into());
        let bookmarks = get_bookmarks(&fx.pool, uri_query, title_query, Vec::new(), 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let title_query = Some("crate".into());
        let tags_query = vec!["error-handling".into()];
        let bookmarks = get_bookmarks(&fx.pool, None, title_query, tags_query, 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let uri_query = Some("github".into());
        let title_query = Some("page".into());
        let tags_query = vec!["sql".into(), "crate".into()];
        let bookmarks = get_bookmarks(&fx.pool, uri_query, title_query, tags_query, 10)
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
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let uri_query = Some("github.com".into());
        let bookmarks = get_bookmarks(&fx.pool, uri_query, None, Vec::new(), 2)
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
        let fx = DBPoolFixture::new().await;

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let now = since_the_epoch.as_secs() as i64;

        create_or_update_bookmark(
            &fx.pool,
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
            &fx.pool,
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
            &fx.pool,
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
            &fx.pool,
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
        let bookmarks = get_bookmarks(&fx.pool, uri_query, None, Vec::new(), 10)
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
        let fx = DBPoolFixture::new().await;
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
                &fx.pool,
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
            let bookmarks = get_bookmarks_by_query(&fx.pool, &query, 10).await.unwrap();

            // THEN
            assert_eq!(
                bookmarks.len(),
                expected_num_bookmarks,
                "failed for query: {:?}",
                query
            );
        }
    }

    #[tokio::test]
    async fn searching_bookmarks_by_query_returns_all_data_for_each_bookmark() {
        // GIVEN
        let fx = DBPoolFixture::new().await;
        save_test_bookmarks(&fx.pool).await;

        // WHEN
        let search_terms = SearchTerms::try_from("crate page").unwrap();
        let bookmarks = get_bookmarks_by_query(&fx.pool, &search_terms, 10)
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
        let tags = get_tags(&fx.pool)
            .await
            .expect("tags should've been fetched");

        // THEN
        assert_yaml_snapshot!(tags, @"
        - tag1
        - tag2
        - tag3
        - tag4
        - tag5
        ");
    }

    #[tokio::test]
    async fn getting_tags_with_stats_works() {
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
        let tags = get_tags_with_stats(&fx.pool)
            .await
            .expect("tags should've been fetched");

        // THEN
        assert_yaml_snapshot!(tags, @"
        - name: tag1
          num_bookmarks: 1
        - name: tag2
          num_bookmarks: 3
        - name: tag3
          num_bookmarks: 4
        - name: tag4
          num_bookmarks: 1
        - name: tag5
          num_bookmarks: 1
        ");
    }

    #[test]
    fn literal_like_substring_pattern_escapes_like_metacharacters() {
        assert_eq!(
            literal_like_substring_pattern(r"100%_literal\value"),
            r"%100\%\_literal\\value%"
        );
    }
}
