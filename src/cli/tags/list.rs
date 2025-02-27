use super::super::DisplayError;
use super::super::display::{display_tags, display_tags_with_stats};
use crate::args::OutputFormat;
use crate::persistence::DBError;
use crate::persistence::{get_tags, get_tags_with_stats};
use crate::tui::{AppTuiError, TuiContext, run_tui};
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum ListTagsError {
    #[error("couldn't get tags from db: {0}")]
    CouldntGetTagsFromDB(#[from] DBError),
    #[error("couldn't display results: {0}")]
    CouldntDisplayResults(#[from] DisplayError),
    #[error(transparent)]
    CouldntRunTui(#[from] AppTuiError),
}

pub async fn list_tags(
    pool: &Pool<Sqlite>,
    format: OutputFormat,
    show_stats: bool,
    tui: bool,
) -> Result<(), ListTagsError> {
    if tui {
        run_tui(pool, TuiContext::Tags).await?;
        return Ok(());
    }

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
