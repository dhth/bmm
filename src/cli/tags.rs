use super::display::{display_tags, display_tags_with_stats};
use super::DisplayError;
use crate::args::OutputFormat;
use crate::persistence::DBError;
use crate::persistence::{get_tags, get_tags_with_stats};
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum ListTagsError {
    #[error("couldn't get tags from db: {0}")]
    CouldntGetTagsFromDB(#[from] DBError),
    #[error("couldn't display results: {0}")]
    CouldntDisplayResults(#[from] DisplayError),
}

pub async fn list_tags(
    pool: &Pool<Sqlite>,
    format: OutputFormat,
    show_stats: bool,
) -> Result<(), ListTagsError> {
    match show_stats {
        true => {
            let tags_stats = get_tags_with_stats(pool).await?;

            display_tags_with_stats(&tags_stats, &format)?;
        }
        false => {
            let tags = get_tags(pool).await?;

            display_tags(&tags, &format)?;
        }
    }

    Ok(())
}
