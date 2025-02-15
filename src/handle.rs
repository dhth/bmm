use crate::args::{Args, BmmCommand, TagsCommand};
use crate::cli::*;
use crate::errors::AppError;
use crate::persistence::get_db_pool;
use dirs::data_dir;
use std::fs;
use std::path::PathBuf;

const DATA_DIR: &str = "bmm";
const DATA_FILE: &str = "bmm.db";

pub async fn handle(args: Args) -> Result<(), AppError> {
    let db_path = match &args.db_path {
        Some(p) => PathBuf::from(p),
        None => {
            let user_data_dir = data_dir().ok_or(AppError::CouldntGetDataDirectory)?;
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

        BmmCommand::Import { file, dry_run } => {
            let result = import_bookmarks(&pool, &file, dry_run).await?;
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
            query,
            format,
            limit,
        } => search_bookmarks(&pool, &query, format, limit)
            .await
            .map_err(AppError::CouldntListBookmarks)?,

        BmmCommand::Save {
            uri,
            title,
            tags,
            use_editor,
            fail_if_uri_already_saved: fail_if_uri_saved,
        } => save_bookmark(&pool, uri, title, tags, use_editor, fail_if_uri_saved).await?,

        BmmCommand::Show { uri } => show_bookmark(&pool, uri).await?,

        BmmCommand::Tags { tags_command } => match tags_command {
            TagsCommand::List { format, show_stats } => {
                list_tags(&pool, format, show_stats).await?
            }
            TagsCommand::Rename {
                original_tag,
                new_tag,
            } => rename_tag(&pool, original_tag, new_tag).await?,
        },
    }

    Ok(())
}
