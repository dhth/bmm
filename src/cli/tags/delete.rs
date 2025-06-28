use crate::persistence::DBError;
use crate::persistence::{delete_tags_by_name, get_tags};
use sqlx::{Pool, Sqlite};
use std::collections::HashSet;
use std::io::{Error as IOError, Write};

#[derive(thiserror::Error, Debug)]
pub enum DeleteTagsError {
    #[error("couldn't flush stdout: {0}")]
    CouldntFlushStdout(IOError),
    #[error("couldn't read your input: {0}")]
    CouldntReadUserInput(IOError),
    #[error("couldn't check if tags exist: {0}")]
    CouldntCheckIfTagsExist(DBError),
    #[error("tags do not exist: {0:?}")]
    TagsDoNotExist(Vec<String>),
    #[error(transparent)]
    CouldntDeleteTags(DBError),
}

pub async fn delete_tags(
    pool: &Pool<Sqlite>,
    tags: Vec<String>,
    skip_confirmation: bool,
) -> Result<(), DeleteTagsError> {
    if tags.is_empty() {
        return Ok(());
    }

    if !skip_confirmation {
        if tags.len() == 1 {
            println!("Deleting 1 tag; enter \"y\" to confirm.");
        } else {
            println!("Deleting {} tags; enter \"y\" to confirm.", tags.len());
        }

        std::io::stdout()
            .flush()
            .map_err(DeleteTagsError::CouldntFlushStdout)?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(DeleteTagsError::CouldntReadUserInput)?;

        if input.trim() != "y" {
            return Ok(());
        }
    }

    let all_tags = get_tags(pool)
        .await
        .map_err(DeleteTagsError::CouldntCheckIfTagsExist)?;

    let non_existent_tags = get_set_difference(&tags, &all_tags);

    if !non_existent_tags.is_empty() {
        return Err(DeleteTagsError::TagsDoNotExist(non_existent_tags));
    }

    let num_tags_deleted = delete_tags_by_name(pool, &tags)
        .await
        .map_err(DeleteTagsError::CouldntDeleteTags)?;

    match num_tags_deleted {
        1 => println!("deleted 1 tag"),
        n => println!("deleted {n} tags"),
    }

    Ok(())
}

fn get_set_difference(smaller: &[String], larger: &[String]) -> Vec<String> {
    let set: HashSet<_> = larger.iter().collect();
    smaller
        .iter()
        .filter(|item| !set.contains(item))
        .cloned()
        .collect()
}
