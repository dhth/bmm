mod save;
mod session;

pub(crate) use save::{
    SaveBookmarkEditorError, SaveBookmarkEditorOutcome, get_save_bookmark_input,
};
pub(crate) use session::EditorError;

const BMM_BANNER: &str = include_str!("assets/banner.txt");
