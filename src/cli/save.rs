use crate::domain::{DraftBookmark, DraftBookmarkError, PotentialBookmark};
use crate::editor::{SaveBookmarkEditorError, SaveBookmarkEditorOutcome, get_save_bookmark_input};
use crate::persistence::{
    DBError, SaveBookmarkOptions, create_or_update_bookmark, get_bookmark_with_exact_uri,
};
use sqlx::{Pool, Sqlite};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(thiserror::Error, Debug)]
pub enum SaveBookmarkError {
    #[error("couldn't check if uri already saved: {0}")]
    CouldntCheckIfBookmarkExists(DBError),
    #[error("uri already saved")]
    UriAlreadySaved,
    #[error(transparent)]
    CouldntUseTextEditor(#[from] SaveBookmarkEditorError),
    #[error(transparent)]
    BookmarkDetailsAreInvalid(#[from] DraftBookmarkError),
    #[error(transparent)]
    CouldntSaveBookmark(DBError),
    #[error("something unexpected happened: {0}")]
    UnexpectedError(String),
}

pub async fn save_bookmark(
    pool: &Pool<Sqlite>,
    potential_bookmark: PotentialBookmark,
    use_editor: bool,
    fail_if_uri_saved: bool,
    reset_missing: bool,
    ignore_attribute_errors: bool,
) -> Result<(), SaveBookmarkError> {
    let maybe_existing_bookmark = get_bookmark_with_exact_uri(pool, &potential_bookmark.uri)
        .await
        .map_err(SaveBookmarkError::CouldntCheckIfBookmarkExists)?;

    if fail_if_uri_saved && maybe_existing_bookmark.is_some() {
        return Err(SaveBookmarkError::UriAlreadySaved);
    }

    if maybe_existing_bookmark.is_some()
        && !use_editor
        && potential_bookmark.title.is_none()
        && potential_bookmark.tags.is_empty()
    {
        println!("nothing to update!");
        return Ok(());
    }

    let potential_bookmark = if use_editor {
        let initial_title = maybe_existing_bookmark
            .as_ref()
            .and_then(|bookmark| bookmark.title.as_deref());
        let initial_tags = maybe_existing_bookmark
            .as_ref()
            .and_then(|bookmark| bookmark.tags.as_deref());

        match get_save_bookmark_input(&potential_bookmark.uri, initial_title, initial_tags)? {
            SaveBookmarkEditorOutcome::Unchanged => return Ok(()),
            SaveBookmarkEditorOutcome::Submitted(bookmark) => bookmark,
        }
    } else {
        potential_bookmark
    };
    let draft_bookmark = DraftBookmark::try_from((potential_bookmark, ignore_attribute_errors))?;

    let reset_missing = if use_editor { true } else { reset_missing };

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SaveBookmarkError::UnexpectedError(format!("system time error: {e}")))?;
    let now = since_the_epoch.as_secs() as i64;
    let save_options = SaveBookmarkOptions {
        reset_missing_attributes: reset_missing,
        reset_tags: reset_missing,
    };
    create_or_update_bookmark(pool, &draft_bookmark, now, save_options)
        .await
        .map_err(SaveBookmarkError::CouldntSaveBookmark)?;

    Ok(())
}
