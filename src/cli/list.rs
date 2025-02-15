use super::display::display_bookmarks;
use super::DisplayError;
use crate::args::OutputFormat;
use crate::persistence::DBError;
use crate::persistence::{get_bookmarks, get_bookmarks_by_query};
use crate::tui::run;
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum ListBookmarksError {
    #[error("couldn't get bookmarks from db: {0}")]
    CouldntGetBookmarksFromDB(DBError),
    #[error("couldn't display results: {0}")]
    CouldntDisplayResults(DisplayError),
}

pub async fn list_bookmarks(
    pool: &Pool<Sqlite>,
    uri: Option<String>,
    title: Option<String>,
    tags: Vec<String>,
    format: OutputFormat,
    limit: u16,
    tui: bool,
) -> Result<(), ListBookmarksError> {
    let bookmarks = get_bookmarks(pool, uri, title, tags, limit)
        .await
        .map_err(ListBookmarksError::CouldntGetBookmarksFromDB)?;

    match tui {
        true => run(bookmarks),
        false => display_bookmarks(&bookmarks, &format)
            .map_err(ListBookmarksError::CouldntDisplayResults)?,
    }

    Ok(())
}

pub async fn search_bookmarks(
    pool: &Pool<Sqlite>,
    query: &str,
    format: OutputFormat,
    limit: u16,
    tui: bool,
) -> Result<(), ListBookmarksError> {
    let bookmarks = get_bookmarks_by_query(pool, query, limit)
        .await
        .map_err(ListBookmarksError::CouldntGetBookmarksFromDB)?;
    match tui {
        true => run(bookmarks),
        false => display_bookmarks(&bookmarks, &format)
            .map_err(ListBookmarksError::CouldntDisplayResults)?,
    }

    Ok(())
}
