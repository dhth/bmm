use crate::cli::{
    CouldntGetDetailsViaEditorError, DeleteBookmarksError, DeleteTagsError, ImportError,
    ListBookmarksError, ListTagsError, ParsingTempFileContentError, RenameTagError,
    SaveBookmarkError, SaveBookmarksError, SearchBookmarksError, ShowBookmarkError,
};
use crate::common::{ENV_VAR_BMM_EDITOR, ENV_VAR_EDITOR, IMPORT_FILE_FORMATS};
use crate::persistence::DBError;
use crate::tui::AppTuiError;
use crate::utils::DataDirError;
use std::io::Error as IOError;

const IMPORT_EXAMPLE_JSON: &str = include_str!("static/import-example.json");
const IGNORE_ERRORS_MESSAGE: &str = "Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.";

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    // data related
    #[error(transparent)]
    CouldntGetDataDirectory(DataDirError),
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
    #[error("couldn't search bookmarks: {0}")]
    CouldntSearchBookmarks(#[from] SearchBookmarksError),
    #[error("couldn't save bookmark: {0}")]
    CouldntSaveBookmark(#[from] SaveBookmarkError),
    #[error("couldn't save bookmarks: {0}")]
    CouldntSaveBookmarks(#[from] SaveBookmarksError),
    #[error("couldn't show bookmark details: {0}")]
    CouldntShowBookmark(#[from] ShowBookmarkError),
    #[error("couldn't delete bookmarks: {0}")]
    CouldntDeleteBookmarks(#[from] DeleteBookmarksError),

    // tags related
    #[error("couldn't list tags: {0}")]
    CouldntListTags(#[from] ListTagsError),
    #[error("couldn't rename tag: {0}")]
    CouldntRenameTag(#[from] RenameTagError),
    #[error("couldn't delete tag(s): {0}")]
    CouldntDeleteTag(#[from] DeleteTagsError),

    // tui related
    #[error("couldn't run bmm's TUI: {0}")]
    CouldntRunTui(#[from] AppTuiError),
}

impl AppError {
    pub fn code(&self) -> Option<u16> {
        match self {
            AppError::CouldntGetDataDirectory(e) => match e {
                #[cfg(target_family = "unix")]
                DataDirError::XDGDataHomeNotAbsolute => None,
                DataDirError::CouldntGetDataDir => Some(100),
            },
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
                    CouldntGetDetailsViaEditorError::CouldntFindEditorExe(..) => None,
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
                SaveBookmarkError::CouldntFetchDetails(_) => None,
                SaveBookmarkError::NoDetailsFetched => None,
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
            AppError::CouldntDeleteTag(e) => match e {
                DeleteTagsError::CouldntFlushStdout(_) => Some(1000),
                DeleteTagsError::CouldntReadUserInput(_) => Some(1001),
                DeleteTagsError::CouldntCheckIfTagsExist(_) => Some(1002),
                DeleteTagsError::TagsDoNotExist(_) => None,
                DeleteTagsError::CouldntDeleteTags(_) => Some(1003),
            },
            AppError::CouldntSaveBookmarks(e) => match e {
                SaveBookmarksError::CouldntReadStdin(_) => Some(2001),
                SaveBookmarksError::TooManyBookmarks(_) => None,
                SaveBookmarksError::ValidationError { .. } => None,
                SaveBookmarksError::SaveError(_) => Some(2002),
                SaveBookmarksError::UnexpectedError(_) => Some(2003),
            },
            AppError::CouldntSearchBookmarks(e) => match e {
                SearchBookmarksError::SearchQueryInvalid(_) => None,
                SearchBookmarksError::CouldntGetBookmarksFromDB(_) => Some(3000),
                SearchBookmarksError::CouldntDisplayResults(_) => Some(3001),
                SearchBookmarksError::CouldntRunTui(e) => Some(e.code()),
            },
        }
    }

    pub fn follow_up(&self) -> Option<String> {
        match self {
            AppError::CouldntGetDataDirectory(e) => match e {
                #[cfg(target_family = "unix")]
                DataDirError::XDGDataHomeNotAbsolute =>
                    Some("Context: XDG specifications dictate that XDG_DATA_HOME must be an absolute path.
Read more here: https://specifications.freedesktop.org/basedir-spec/latest/#basics".into()),
                DataDirError::CouldntGetDataDir =>
                    Some("Possible workaround: manually specify the path for bmm's database using --db-path".into())
            },
            AppError::CouldntImportBookmarks(e) => match e {
                ImportError::FileHasNoExtension => Some(format!("bmm can only import from files with one of these extensions: {:?}", IMPORT_FILE_FORMATS)),
                ImportError::ValidationError { .. } => Some(IGNORE_ERRORS_MESSAGE.into()),
                ImportError::CouldntDeserializeJSONInput(_) =>
                    Some(format!("Suggestion: ensure the file is valid JSON and looks like the following:

{}", IMPORT_EXAMPLE_JSON )),
                _ => None,
            },
            AppError::CouldntSaveBookmark(e) => match e {
                SaveBookmarkError::BookmarkDetailsAreInvalid(_) => Some(IGNORE_ERRORS_MESSAGE.into()),
                SaveBookmarkError::CouldntUseTextEditor(se) => match se {
                    CouldntGetDetailsViaEditorError::CouldntFindEditorExe(editor_exe, env_var_used, _) =>
                        Some(format!(r#"Context: bmm used the environment variable {} to determine your text editor.
Check if "{}" actually points to your text editor's executable."#, env_var_used, editor_exe)),
                    CouldntGetDetailsViaEditorError::NoEditorConfigured =>
                        Some(format!("Suggestion: set the environment variables {} or {} to use this feature", ENV_VAR_BMM_EDITOR, ENV_VAR_EDITOR)),
                    CouldntGetDetailsViaEditorError::ParsingEditorText(ParsingTempFileContentError::InputMissing) =>
                        Some("Suggestion: enter the details between the >>>/<<< markers without changing the structure of the document".into()),
                        _ => None,
                },
                SaveBookmarkError::UnexpectedError(_) => None,
                _ => None,
            },
            AppError::CouldntSaveBookmarks(SaveBookmarksError::ValidationError { .. }) => Some(IGNORE_ERRORS_MESSAGE.into()),
            _ => None,
        }
    }
}
