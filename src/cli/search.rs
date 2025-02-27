use super::DisplayError;
use super::display::display_bookmarks;
use crate::args::OutputFormat;
use crate::persistence::DBError;
use crate::persistence::{SearchTerms, SearchTermsError, get_bookmarks_by_query};
use crate::tui::run_tui;
use crate::tui::{AppTuiError, TuiContext};
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum SearchBookmarksError {
    #[error("search query is invalid: {0}")]
    SearchQueryInvalid(#[from] SearchTermsError),
    #[error("couldn't get bookmarks from db: {0}")]
    CouldntGetBookmarksFromDB(DBError),
    #[error("couldn't display results: {0}")]
    CouldntDisplayResults(DisplayError),
    #[error(transparent)]
    CouldntRunTui(#[from] AppTuiError),
}

pub async fn search_bookmarks(
    pool: &Pool<Sqlite>,
    query_terms: &Vec<String>,
    format: OutputFormat,
    limit: u16,
    tui: bool,
) -> Result<(), SearchBookmarksError> {
    let search_terms = SearchTerms::try_from(query_terms)?;

    if tui {
        run_tui(pool, TuiContext::Search(search_terms)).await?;
        return Ok(());
    }

    let bookmarks = get_bookmarks_by_query(pool, &search_terms, limit)
        .await
        .map_err(SearchBookmarksError::CouldntGetBookmarksFromDB)?;

    if bookmarks.is_empty() {
        return Ok(());
    }

    display_bookmarks(&bookmarks, &format).map_err(SearchBookmarksError::CouldntDisplayResults)?;

    Ok(())
}
