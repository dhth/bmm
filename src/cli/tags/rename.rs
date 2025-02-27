use crate::domain::{TAG_REGEX_STR, Tag};
use crate::persistence::DBError;
use crate::persistence::rename_tag_name;
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum RenameTagError {
    #[error("source and target tag are the same")]
    SourceAndTargetSame,
    #[error("no such tag")]
    NoSuchTag,
    #[error(transparent)]
    CouldntRenameTag(#[from] DBError),
    #[error("new tag is invalid (valid regex: {TAG_REGEX_STR})")]
    TagIsInvalid,
}

pub async fn rename_tag(
    pool: &Pool<Sqlite>,
    source_tag: String,
    target_tag: String,
) -> Result<(), RenameTagError> {
    if source_tag.trim() == target_tag.trim() {
        return Err(RenameTagError::SourceAndTargetSame);
    }

    let new_tag = Tag::try_from(target_tag.as_str()).map_err(|_| RenameTagError::TagIsInvalid)?;
    let result = rename_tag_name(pool, source_tag, new_tag).await?;
    if result == 0 {
        return Err(RenameTagError::NoSuchTag);
    }

    Ok(())
}
