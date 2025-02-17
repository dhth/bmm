use super::common::ActivePane;
use crate::domain::SavedBookmark;
use crate::persistence::DBError;
use ratatui::crossterm::event::Event;
use std::io::Error as IOError;

pub enum Message {
    TerminalResize(u16, u16),
    ClearUserMsg,
    GoToNextItem,
    GoToPreviousItem,
    GoToFirstItem,
    GoToLastItem,
    OpenInBrowser,
    UrlsOpenedInBrowser(UrlsOpenedResult),
    SearchFinished(Result<Vec<SavedBookmark>, DBError>),
    ShowView(ActivePane),
    SearchInputGotEvent(Event),
    SubmitSearch,
    GoBackOrQuit,
}

pub enum UrlsOpenedResult {
    Success,
    Failure(IOError),
}
