use super::display::display_bookmarks;
use super::DisplayError;
use crate::args::OutputFormat;
use crate::persistence::get_bookmarks;
use crate::persistence::DBError;
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
) -> Result<(), ListBookmarksError> {
    let bookmarks = get_bookmarks(pool, uri, title, tags, limit)
        .await
        .map_err(ListBookmarksError::CouldntGetBookmarksFromDB)?;

    if bookmarks.is_empty() {
        return Ok(());
    }

    display_bookmarks(&bookmarks, &format).map_err(ListBookmarksError::CouldntDisplayResults)?;

    Ok(())
}
