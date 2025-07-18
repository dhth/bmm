use crate::persistence::DBError;
use crate::persistence::delete_bookmarks_with_uris;
use sqlx::{Pool, Sqlite};
use std::io::{Error as IOError, Write};

#[derive(thiserror::Error, Debug)]
pub enum DeleteBookmarksError {
    #[error(transparent)]
    CouldntDeleteBookmarksInDB(#[from] DBError),
    #[error("couldn't flush stdout: {0}")]
    CouldntFlushStdout(IOError),
    #[error("couldn't read your input: {0}")]
    CouldntReadUserInput(IOError),
}

pub async fn delete_bookmarks(
    pool: &Pool<Sqlite>,
    uris: Vec<String>,
    skip_confirmation: bool,
) -> Result<(), DeleteBookmarksError> {
    if uris.is_empty() {
        return Ok(());
    }

    if !skip_confirmation {
        if uris.len() == 1 {
            println!("Deleting 1 bookmark; enter \"y\" to confirm.");
        } else {
            println!("Deleting {} bookmarks; enter \"y\" to confirm.", uris.len());
        }

        std::io::stdout()
            .flush()
            .map_err(DeleteBookmarksError::CouldntFlushStdout)?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(DeleteBookmarksError::CouldntReadUserInput)?;

        if input.trim() != "y" {
            return Ok(());
        }
    }

    let num_bookmarks = delete_bookmarks_with_uris(pool, &uris).await?;

    match num_bookmarks {
        0 => println!("nothing got deleted"),
        1 => println!("deleted 1 bookmark"),
        n => println!("deleted {n} bookmarks"),
    }

    Ok(())
}
