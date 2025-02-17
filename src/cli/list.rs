use super::display::display_bookmarks;
use super::DisplayError;
use crate::args::OutputFormat;
use crate::persistence::DBError;
use crate::persistence::{get_bookmarks, get_bookmarks_by_query};
use crate::tui::run_tui;
use crate::tui::AppTuiError;
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum ListBookmarksError {
    #[error("couldn't get bookmarks from db: {0}")]
    CouldntGetBookmarksFromDB(DBError),
    #[error("couldn't display results: {0}")]
    CouldntDisplayResults(DisplayError),
    #[error(transparent)]
    CouldntRunTui(#[from] AppTuiError),
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

    if bookmarks.is_empty() {
        return Ok(());
    }

    match tui {
        true => run_tui(pool, bookmarks).await?,
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

    if bookmarks.is_empty() {
        return Ok(());
    }

    match tui {
        true => run_tui(pool, bookmarks).await?,
        false => display_bookmarks(&bookmarks, &format)
            .map_err(ListBookmarksError::CouldntDisplayResults)?,
    }

    Ok(())
}
