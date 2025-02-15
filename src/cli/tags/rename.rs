use crate::domain::{Tag, TAG_REGEX_STR};
use crate::persistence::rename_tag_name;
use crate::persistence::DBError;
use sqlx::{Pool, Sqlite};

#[derive(thiserror::Error, Debug)]
pub enum RenameTagError {
    #[error("new tag and original tag are the same")]
    NewTagNotDifferent,
    #[error("no such tag")]
    NoSuchTag,
    #[error(transparent)]
    CouldntRenameTag(#[from] DBError),
    #[error("new tag is invalid (valid regex: {TAG_REGEX_STR})")]
    TagIsInvalid,
}

pub async fn rename_tag(
    pool: &Pool<Sqlite>,
    original_tag: String,
    new_tag: String,
) -> Result<(), RenameTagError> {
    if original_tag.trim() == new_tag.trim() {
        return Err(RenameTagError::NewTagNotDifferent);
    }

    let new_tag = Tag::try_from(new_tag.as_str()).map_err(|_| RenameTagError::TagIsInvalid)?;
    let result = rename_tag_name(pool, original_tag, new_tag).await?;
    if result == 0 {
        return Err(RenameTagError::NoSuchTag);
    }

    Ok(())
}
