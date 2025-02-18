use super::common::ActivePane;
use crate::domain::{SavedBookmark, TagStats};
use crate::persistence::DBError;
use ratatui::crossterm::event::Event;
use std::io::Error as IOError;

pub enum Message {
    TerminalResize(u16, u16),
    ClearUserMsg,
    GoToNextListItem,
    GoToPreviousListItem,
    GoToFirstListItem,
    GoToLastListItem,
    OpenInBrowser,
    UrlsOpenedInBrowser(UrlsOpenedResult),
    SearchFinished(Result<Vec<SavedBookmark>, DBError>),
    TagsFetched(Result<Vec<TagStats>, DBError>),
    ShowView(ActivePane),
    SearchInputGotEvent(Event),
    SubmitSearch,
    ShowBookmarksForTag,
    BookmarksForTagFetched(Result<Vec<SavedBookmark>, DBError>),
    GoBackOrQuit,
}

pub enum UrlsOpenedResult {
    Success,
    Failure(IOError),
}
