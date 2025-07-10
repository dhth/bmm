use super::common::ActivePane;
use super::model::Model;
use crate::domain::{SavedBookmark, TagStats};
use crate::persistence::DBError;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use std::io::Error as IOError;

pub enum Message {
    TerminalResize(u16, u16),
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
    CopyURIToClipboard,
    CopyURIsToClipboard,
    SubmitSearch,
    ShowBookmarksForTag,
    BookmarksForTagFetched(Result<Vec<SavedBookmark>, DBError>),
    ContentCopiedToClipboard(Result<(), String>),
    GoBackOrQuit,
}

pub enum UrlsOpenedResult {
    Success,
    Failure(IOError),
}

pub fn get_event_handling_msg(model: &Model, event: Event) -> Option<Message> {
    match event {
        Event::Key(key_event) => match model.terminal_too_small {
            true => match key_event.kind {
                KeyEventKind::Press => match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => Some(Message::GoBackOrQuit),
                    _ => None,
                },
                _ => None,
            },
            false => match key_event.kind {
                KeyEventKind::Press => match model.active_pane {
                    ActivePane::List => match key_event.code {
                        KeyCode::Char('j') | KeyCode::Down => Some(Message::GoToNextListItem),
                        KeyCode::Char('k') | KeyCode::Up => Some(Message::GoToPreviousListItem),
                        KeyCode::Char('g') => Some(Message::GoToFirstListItem),
                        KeyCode::Char('G') => Some(Message::GoToLastListItem),
                        KeyCode::Char('o') => Some(Message::OpenInBrowser),
                        KeyCode::Char('s') => Some(Message::ShowView(ActivePane::SearchInput)),
                        KeyCode::Char('t') | KeyCode::Tab => {
                            Some(Message::ShowView(ActivePane::TagsList))
                        }
                        KeyCode::Char('y') => Some(Message::CopyURIToClipboard),
                        KeyCode::Char('Y') => Some(Message::CopyURIsToClipboard),
                        KeyCode::Esc | KeyCode::Char('q') => Some(Message::GoBackOrQuit),
                        KeyCode::Char('?') => Some(Message::ShowView(ActivePane::Help)),
                        _ => None,
                    },
                    ActivePane::Help => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => Some(Message::GoBackOrQuit),
                        KeyCode::Char('?') => Some(Message::ShowView(ActivePane::List)),
                        _ => None,
                    },
                    ActivePane::SearchInput => match key_event.code {
                        KeyCode::Esc => Some(Message::GoBackOrQuit),
                        KeyCode::Enter => Some(Message::SubmitSearch),
                        KeyCode::Down => Some(Message::GoToNextListItem),
                        KeyCode::Up => Some(Message::GoToPreviousListItem),
                        _ => Some(Message::SearchInputGotEvent(event)),
                    },
                    ActivePane::TagsList => match key_event.code {
                        KeyCode::Char('j') | KeyCode::Down => Some(Message::GoToNextListItem),
                        KeyCode::Char('k') | KeyCode::Up => Some(Message::GoToPreviousListItem),
                        KeyCode::Char('g') => Some(Message::GoToFirstListItem),
                        KeyCode::Char('G') => Some(Message::GoToLastListItem),
                        KeyCode::Enter => Some(Message::ShowBookmarksForTag),
                        KeyCode::Esc | KeyCode::Char('q') => Some(Message::GoBackOrQuit),
                        _ => None,
                    },
                },
                _ => None,
            },
        },
        Event::Resize(w, h) => Some(Message::TerminalResize(w, h)),
        _ => None,
    }
}
