use super::{
    commands::Command,
    common::{ActivePane, COLOR_TWO},
};
use crate::domain::{SavedBookmark, TagStats};
use ratatui::{
    style::Style,
    text::Line,
    widgets::{ListItem, ListState},
};
use sqlx::{Pool, Sqlite};
use std::time::Instant;
use tui_input::Input;

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
pub(crate) struct TagItems {
    pub(crate) items: Vec<TagStats>,
    pub(crate) state: ListState,
}

#[derive(Debug)]
pub(crate) enum UserMessage {
    Info(String, Instant),
    Error(String, Instant),
}

impl UserMessage {
    pub(super) fn info(message: &str) -> Self {
        UserMessage::Info(message.to_string(), Instant::now())
    }
    pub(super) fn error(message: &str) -> Self {
        UserMessage::Error(message.to_string(), Instant::now())
    }
}

pub(super) enum TerminalDimensions {
    Unknown,
    Known(u16, u16),
}

pub enum TuiContext {
    Initial,
    Search(String),
    Tags,
}

impl BookmarkItems {
    fn default() -> Self {
        let state = ListState::default().with_selected(None);

        Self {
            items: vec![],
            state,
        }
    }
}

impl TagItems {
    fn default() -> Self {
        let state = ListState::default().with_selected(None);

        Self {
            items: vec![],
            state,
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

impl From<(Vec<SavedBookmark>, usize)> for BookmarkItems {
    fn from(value: (Vec<SavedBookmark>, usize)) -> Self {
        let bookmarks = value.0;
        let index = value.1;
        let items = bookmarks
            .into_iter()
            .map(|bookmark| BookmarkItem::new(bookmark, false))
            .collect();
        let state = ListState::default().with_selected(Some(index));

        Self { items, state }
    }
}

impl From<Vec<TagStats>> for TagItems {
    fn from(tags: Vec<TagStats>) -> Self {
        let state = ListState::default().with_selected(Some(0));

        Self { items: tags, state }
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
                Style::new().fg(COLOR_TWO),
            ),
        };
        ListItem::new(line)
    }
}

impl From<&TagStats> for ListItem<'_> {
    fn from(tag_with_stats: &TagStats) -> Self {
        let line = Line::from(tag_with_stats.name.clone());
        ListItem::new(line)
    }
}

pub(super) struct Model {
    pub pool: Pool<Sqlite>,
    pub active_pane: ActivePane,
    pub(super) bookmark_items: BookmarkItems,
    pub(super) tag_items: TagItems,
    pub(super) running_state: RunningState,
    pub(super) user_message: Option<UserMessage>,
    pub(super) render_counter: u64,
    pub(super) event_counter: u64,
    pub(super) search_input: Input,
    pub(super) initial: bool,
    pub(super) terminal_dimensions: TerminalDimensions,
    pub(super) terminal_too_small: bool,
    pub(super) debug: bool,
}

impl Model {
    pub(crate) fn default(pool: &Pool<Sqlite>, context: TuiContext) -> Self {
        let debug = std::env::var("BMM_DEBUG").unwrap_or_default().trim() == "1";

        let active_pane = match context {
            TuiContext::Search(_) => ActivePane::List,
            TuiContext::Tags => ActivePane::TagsList,
            TuiContext::Initial => ActivePane::SearchInput,
        };

        let initial = matches!(context, TuiContext::Initial);

        Self {
            pool: pool.clone(),
            active_pane,
            running_state: RunningState::Running,
            bookmark_items: BookmarkItems::default(),
            tag_items: TagItems::default(),
            user_message: None,
            render_counter: 0,
            event_counter: 0,
            search_input: Input::default(),
            initial,
            terminal_dimensions: TerminalDimensions::Unknown,
            terminal_too_small: false,
            debug,
        }
    }

    pub(super) fn select_next_list_item(&mut self) {
        match self.active_pane {
            ActivePane::List => self.bookmark_items.state.select_next(),
            ActivePane::TagsList => self.tag_items.state.select_next(),
            ActivePane::SearchInput => {}
            ActivePane::Help => {}
        }
    }

    pub(super) fn select_previous_list_item(&mut self) {
        match self.active_pane {
            ActivePane::List => self.bookmark_items.state.select_previous(),
            ActivePane::TagsList => self.tag_items.state.select_previous(),
            ActivePane::SearchInput => {}
            ActivePane::Help => {}
        }
    }

    pub(super) fn select_first_list_item(&mut self) {
        match self.active_pane {
            ActivePane::List => self.bookmark_items.state.select_first(),
            ActivePane::TagsList => self.tag_items.state.select_first(),
            ActivePane::SearchInput => {}
            ActivePane::Help => {}
        }
    }
    pub(super) fn select_last_list_item(&mut self) {
        match self.active_pane {
            ActivePane::List => self.bookmark_items.state.select_last(),
            ActivePane::TagsList => self.tag_items.state.select_last(),
            ActivePane::SearchInput => {}
            ActivePane::Help => {}
        }
    }

    pub(super) fn show_view(&mut self, view: ActivePane) -> Option<Command> {
        self.active_pane = match self.active_pane {
            ActivePane::Help => ActivePane::List,
            ActivePane::List => view,
            ActivePane::TagsList => view,
            ActivePane::SearchInput => view,
        };

        match view {
            ActivePane::Help => None,
            ActivePane::List => None,
            ActivePane::TagsList => {
                if self.tag_items.items.is_empty() {
                    Some(Command::FetchTags)
                } else {
                    None
                }
            }
            ActivePane::SearchInput => None,
        }
    }

    pub(super) fn go_back_or_quit(&mut self) {
        match self.active_pane {
            ActivePane::List => self.running_state = RunningState::Done,
            ActivePane::Help => self.active_pane = ActivePane::List,
            ActivePane::SearchInput => {
                self.search_input.reset();
                self.active_pane = ActivePane::List;
            }
            ActivePane::TagsList => {
                if self.bookmark_items.items.is_empty() {
                    self.running_state = RunningState::Done;
                } else {
                    self.active_pane = ActivePane::List;
                }
            }
        };
    }

    pub(super) fn get_cmd_to_open_selection_in_browser(&self) -> Option<Command> {
        let url = match self.bookmark_items.state.selected() {
            Some(i) => match self.bookmark_items.items.get(i) {
                Some(bi) => bi.bookmark.uri.clone(),
                None => return None,
            },
            None => return None,
        };

        Some(Command::OpenInBrowser(url))
    }
}
