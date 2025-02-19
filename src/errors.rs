use crate::cli::{
    CouldntGetDetailsViaEditorError, DeleteBookmarksError, ImportError, ListBookmarksError,
    ListTagsError, ParsingTempFileContentError, RenameTagError, SaveBookmarkError,
    ShowBookmarkError,
};
use crate::persistence::DBError;
use crate::tui::AppTuiError;
use std::io::Error as IOError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    // data related
    #[error("couldn't get your data directory, trying passing a db path manually")]
    CouldntGetDataDirectory,
    #[error("could not create data directory: {0}")]
    CouldntCreateDataDirectory(IOError),
    #[error("couldn't initialize bmm's database: {0}")]
    CouldntInitializeDatabase(#[from] DBError),
    #[error("database path is not valid string")]
    DBPathNotValidStr,

    // bookmarks related
    #[error("couldn't import bookmarks: {0}")]
    CouldntImportBookmarks(#[from] ImportError),
    #[error("couldn't list bookmarks: {0}")]
    CouldntListBookmarks(#[from] ListBookmarksError),
    #[error("couldn't save bookmark: {0}")]
    CouldntSaveBookmark(#[from] SaveBookmarkError),
    #[error("couldn't show bookmark details: {0}")]
    CouldntShowBookmark(#[from] ShowBookmarkError),
    #[error("couldn't delete bookmarks: {0}")]
    CouldntDeleteBookmarks(#[from] DeleteBookmarksError),

    // tags related
    #[error("couldn't list tags: {0}")]
    CouldntListTags(#[from] ListTagsError),
    #[error("couldn't rename tag: {0}")]
    CouldntRenameTag(#[from] RenameTagError),

    // tui related
    #[error("couldn't run bmm's TUI: {0}")]
    CouldntRunTui(#[from] AppTuiError),
}

impl AppError {
    pub fn code(&self) -> Option<u16> {
        match self {
            AppError::CouldntGetDataDirectory => Some(100),
            AppError::CouldntCreateDataDirectory(_) => Some(101),
            AppError::CouldntInitializeDatabase(_) => Some(102),
            AppError::DBPathNotValidStr => None,
            AppError::CouldntImportBookmarks(e) => match e {
                ImportError::FileHasNoExtension => None,
                ImportError::FileDoesntExist => None,
                ImportError::CouldntOpenFile(_) => None,
                ImportError::CouldntReadFile(_) => None,
                ImportError::CouldntDeserializeJSONInput(_) => None,
                ImportError::CouldntParseHTMLInput(_) => None,
                ImportError::FileFormatNotSupported(_) => None,
                ImportError::UnexpectedError(_) => Some(300),
                ImportError::TooManyBookmarks(_) => None,
                ImportError::ValidationError { .. } => None,
                ImportError::SaveError(_) => Some(301),
            },
            AppError::CouldntListBookmarks(e) => match e {
                ListBookmarksError::CouldntGetBookmarksFromDB(_) => Some(400),
                ListBookmarksError::CouldntDisplayResults(_) => Some(401),
                ListBookmarksError::CouldntRunTui(e) => Some(e.code()),
            },
            AppError::CouldntSaveBookmark(e) => match e {
                SaveBookmarkError::CouldntCheckIfBookmarkExists(_) => Some(500),
                SaveBookmarkError::UriAlreadySaved => None,
                SaveBookmarkError::BookmarkDetailsAreInvalid(_) => None,
                SaveBookmarkError::CouldntSaveBookmark(_) => Some(501),
                SaveBookmarkError::CouldntUseTextEditor(se) => match se {
                    CouldntGetDetailsViaEditorError::CreateTempFile(_) => Some(550),
                    CouldntGetDetailsViaEditorError::OpenTempFile(_) => Some(551),
                    CouldntGetDetailsViaEditorError::WriteToTempFile(_) => Some(552),
                    CouldntGetDetailsViaEditorError::CouldntFindEditorExe(_) => None,
                    CouldntGetDetailsViaEditorError::OpenTextEditor(_, _) => Some(553),
                    CouldntGetDetailsViaEditorError::ReadTempFileContents(_) => Some(554),
                    CouldntGetDetailsViaEditorError::InvalidEditorEnvVar(_) => None,
                    CouldntGetDetailsViaEditorError::NoEditorConfigured => None,
                    CouldntGetDetailsViaEditorError::ParsingEditorText(pe) => match pe {
                        ParsingTempFileContentError::IncorrectRegexError(_) => Some(560),
                        ParsingTempFileContentError::InputMissing => None,
                    },
                },
                SaveBookmarkError::UnexpectedError(_) => Some(580),
            },
            AppError::CouldntShowBookmark(e) => match e {
                ShowBookmarkError::CouldntGetBookmarkFromDB(_) => Some(600),
                ShowBookmarkError::BookmarkDoesntExist => None,
            },
            AppError::CouldntListTags(e) => match e {
                ListTagsError::CouldntGetTagsFromDB(_) => Some(700),
                ListTagsError::CouldntDisplayResults(_) => Some(701),
                ListTagsError::CouldntRunTui(e) => Some(e.code()),
            },
            AppError::CouldntDeleteBookmarks(e) => match e {
                DeleteBookmarksError::CouldntDeleteBookmarksInDB(_) => Some(800),
                DeleteBookmarksError::CouldntFlushStdout(_) => Some(801),
                DeleteBookmarksError::CouldntReadUserInput(_) => Some(802),
            },
            AppError::CouldntRenameTag(e) => match e {
                RenameTagError::SourceAndTargetSame => None,
                RenameTagError::NoSuchTag => None,
                RenameTagError::CouldntRenameTag(_) => Some(900),
                RenameTagError::TagIsInvalid => None,
            },
            AppError::CouldntRunTui(e) => Some(e.code()),
        }
    }
}
