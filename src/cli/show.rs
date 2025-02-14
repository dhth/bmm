use super::display::display_bookmark_details;
use crate::persistence::get_bookmark_with_exact_uri;
use crate::persistence::DBError;
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum ShowBookmarkError {
    #[error("couldn't get bookmark from db: {0}")]
    CouldntGetBookmarkFromDB(#[from] DBError),
    #[error("bookmark doesn't exist")]
    BookmarkDoesntExist,
}

pub async fn show_bookmark(pool: &Pool<Sqlite>, uri: String) -> Result<(), ShowBookmarkError> {
    let maybe_bookmark = get_bookmark_with_exact_uri(pool, &uri).await?;

    let bookmark = maybe_bookmark.ok_or(ShowBookmarkError::BookmarkDoesntExist)?;

    display_bookmark_details(&bookmark);

    Ok(())
}
