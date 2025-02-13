use crate::cli::{
    CouldntGetDetailsViaEditorError, ImportError, ListBookmarksError, ListTagsError,
    ParsingTempFileContentError, SaveBookmarkError, ShowBookmarkError,
};
use crate::persistence::DBError;
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

    // tags related
    #[error("couldn't list tags: {0}")]
    CouldntListTags(#[from] ListTagsError),
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
            },
        }
    }
}
