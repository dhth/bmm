use super::common::{View, SECONDARY_COLOR};
use crate::domain::SavedBookmark;
use ratatui::{
    style::Style,
    text::Line,
    widgets::{ListItem, ListState},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug)]
pub(crate) struct BookmarkItem {
    pub(crate) bookmark: SavedBookmark,
    pub(crate) status: bool,
}

#[derive(Debug)]
pub(crate) struct BookmarkItems {
    pub(crate) items: Vec<BookmarkItem>,
    pub(crate) state: ListState,
}

#[derive(Debug)]
pub(crate) enum UserMessage {
    Info(String),
    Error(String),
}

pub(super) struct Model {
    pub view: View,
    pub(super) bookmark_items: BookmarkItems,
    pub(super) running_state: RunningState,
    pub(super) message: Option<UserMessage>,
}

impl Model {
    pub(crate) fn default(bookmarks: Vec<SavedBookmark>) -> Self {
        Self {
            view: View::List,
            running_state: RunningState::Running,
            bookmark_items: BookmarkItems::from(bookmarks),
            message: None,
        }
    }
}

impl From<Vec<SavedBookmark>> for BookmarkItems {
    fn from(bookmarks: Vec<SavedBookmark>) -> Self {
        let items = bookmarks
            .into_iter()
            .map(|bookmark| BookmarkItem::new(bookmark, false))
            .collect();
        let state = ListState::default().with_selected(Some(0));

        Self { items, state }
    }
}

impl BookmarkItem {
    fn new(bookmark: SavedBookmark, status: bool) -> Self {
        Self { bookmark, status }
    }
}

impl From<&BookmarkItem> for ListItem<'_> {
    fn from(value: &BookmarkItem) -> Self {
        let line = match value.status {
            false => Line::from(value.bookmark.uri.clone()),
            true => Line::styled(
                format!("> {}", value.bookmark.uri.clone()),
                Style::new().fg(SECONDARY_COLOR),
            ),
        };
        ListItem::new(line)
    }
}
