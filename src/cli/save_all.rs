use crate::common::IMPORT_UPPER_LIMIT;
use crate::domain::{DraftBookmark, DraftBookmarkErrors};
use crate::persistence::{create_or_update_bookmarks, DBError};
use sqlx::{Pool, Sqlite};
use std::io::BufRead;
use std::io::Error as IOError;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(thiserror::Error, Debug)]
pub enum SaveBookmarksError {
    #[error("couldn't read stdin: {0}")]
    CouldntReadStdin(#[from] IOError),
    #[error("file has too many bookmarks: {0} (maximum allowed at a time: {IMPORT_UPPER_LIMIT})")]
    TooManyBookmarks(usize),
    #[error("{}\n\n{}", errors.msg(), errors)]
    ValidationError { errors: DraftBookmarkErrors },
    #[error("couldn't save bookmarks to bmm's database: {0}")]
    SaveError(#[from] DBError),
    #[error("something unexpected happened: {0}")]
    UnexpectedError(String),
}

#[derive(Debug)]
pub struct SaveAllStats {
    pub num_bookmarks: usize,
}

pub async fn save_all_bookmarks(
    pool: &Pool<Sqlite>,
    uris: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    use_stdin: bool,
    reset_missing: bool,
) -> Result<Option<SaveAllStats>, SaveBookmarksError> {
    let mut uris_to_save = uris.unwrap_or_default();

    if use_stdin {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            uris_to_save.push(line?);
        }
    }

    if uris_to_save.len() > IMPORT_UPPER_LIMIT {
        return Err(SaveBookmarksError::TooManyBookmarks(uris_to_save.len()));
    }

    let mut validation_errors = Vec::new();
    let mut draft_bookmarks = Vec::new();

    let tags_to_save = tags.unwrap_or_default();
    let tags_ref = tags_to_save.iter().map(|t| t.as_str()).collect::<Vec<_>>();

    for (index, uri) in uris_to_save.into_iter().enumerate() {
        let db_result = DraftBookmark::try_from((uri.as_str(), None, &tags_ref));
        match db_result {
            Ok(db) => draft_bookmarks.push(db),
            Err(e) => validation_errors.push((index, e)),
        }
    }

    if !validation_errors.is_empty() {
        return Err(SaveBookmarksError::ValidationError {
            errors: DraftBookmarkErrors {
                errors: validation_errors,
            },
        });
    }

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SaveBookmarksError::UnexpectedError(format!("system time error: {}", e)))?;
    let now = since_the_epoch.as_secs() as i64;
    create_or_update_bookmarks(pool, &draft_bookmarks, now, reset_missing).await?;

    Ok(Some(SaveAllStats {
        num_bookmarks: draft_bookmarks.len(),
    }))
}
