use crate::args::{Args, BmmCommand, TagsCommand};
use crate::cli::*;
use crate::domain::PotentialBookmark;
use crate::errors::AppError;
use crate::persistence::get_db_pool;
use crate::tui::{TuiContext, run_tui};
use crate::utils::get_data_dir;
use std::fs;
use std::path::PathBuf;

const DATA_DIR: &str = "bmm";
const DATA_FILE: &str = "bmm.db";

pub async fn handle(args: Args) -> Result<(), AppError> {
    let db_path = match &args.db_path {
        Some(p) => PathBuf::from(p),
        None => {
            let user_data_dir = get_data_dir().map_err(AppError::CouldntGetDataDirectory)?;
            let data_dir = user_data_dir.join(PathBuf::from(DATA_DIR));

            if !data_dir.exists() {
                fs::create_dir_all(&data_dir).map_err(AppError::CouldntCreateDataDirectory)?;
            }

            data_dir.join(PathBuf::from(DATA_FILE))
        }
    };

    let db_path = db_path.to_str().ok_or(AppError::DBPathNotValidStr)?;

    if args.debug {
        display_debug_info(&args, db_path);
        return Ok(());
    }

    let pool = get_db_pool(db_path).await?;

    match args.command {
        BmmCommand::Delete {
            uris,
            skip_confirmation,
        } => {
            delete_bookmarks(&pool, uris, skip_confirmation).await?;
        }

        BmmCommand::Import {
            file,
            reset_missing,
            dry_run,
        } => {
            let result = import_bookmarks(&pool, &file, reset_missing, dry_run).await?;
            if let Some(stats) = result {
                println!("imported {} bookmarks", stats.num_bookmarks_imported);
            }
        }

        BmmCommand::List {
            uri,
            title,
            tags,
            format,
            limit,
        } => list_bookmarks(&pool, uri, title, tags, format, limit).await?,

        BmmCommand::Search {
            query_terms,
            format,
            limit,
            tui,
        } => search_bookmarks(&pool, &query_terms, format, limit, tui).await?,

        BmmCommand::Save {
            uri,
            title,
            tags,
            use_editor,
            fail_if_uri_already_saved,
            reset_missing,
            ignore_attribute_errors,
        } => {
            let potential_bookmark = PotentialBookmark::from((uri, title, &tags));

            save_bookmark(
                &pool,
                potential_bookmark,
                use_editor,
                fail_if_uri_already_saved,
                reset_missing,
                ignore_attribute_errors,
            )
            .await?
        }

        BmmCommand::SaveAll {
            uris,
            tags,
            use_stdin,
            reset_missing,
        } => {
            let result = save_all_bookmarks(&pool, uris, tags, use_stdin, reset_missing).await?;
            if let Some(stats) = result {
                println!("saved {} bookmarks", stats.num_bookmarks);
            }
        }

        BmmCommand::Show { uri } => show_bookmark(&pool, uri).await?,

        BmmCommand::Tags { tags_command } => match tags_command {
            TagsCommand::List {
                format,
                show_stats,
                tui,
            } => list_tags(&pool, format, show_stats, tui).await?,
            TagsCommand::Rename {
                source_tag,
                target_tag,
            } => rename_tag(&pool, source_tag, target_tag).await?,
            TagsCommand::Delete {
                tags,
                skip_confirmation,
            } => delete_tags(&pool, tags, skip_confirmation).await?,
        },
        BmmCommand::Tui => run_tui(&pool, TuiContext::Initial).await?,
    }

    Ok(())
}
