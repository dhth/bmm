use super::commands::Command;
use super::common::*;
use super::handle::handle_command;
use super::message::{Message, UrlsOpenedResult};
use super::model::*;
use super::view::view;
use crate::persistence::SearchTerms;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use sqlx::{Pool, Sqlite};
use std::io::Error as IOError;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tui_input::backend::crossterm::EventHandler;

const EVENT_POLL_DURATION_MS: u64 = 16;

#[derive(thiserror::Error, Debug)]
pub enum AppTuiError {
    #[error("couldn't initialize bmm's TUI: {0}")]
    InitializeTerminal(IOError),
    #[error("couldn't determine terminal size: {0}")]
    DetermineTerminalSize(IOError),
    #[error("couldn't restore terminal to its original state: {0}")]
    RestoreTerminal(IOError),
    #[error("couldn't send a message to internal async queue: {0}")]
    SendMsg(#[from] TrySendError<Message>),
    #[error("couldn't draw a TUI frame: {0}")]
    DrawFrame(IOError),
    #[error("couldn't poll for internal events: {0}")]
    PollForEvents(IOError),
    #[error("couldn't read internal event: {0}")]
    ReadEvent(IOError),
}

pub async fn run_tui(pool: &Pool<Sqlite>, context: TuiContext) -> Result<(), AppTuiError> {
    let mut tui = AppTui::new(pool, context)?;
    tui.run().await?;

    Ok(())
}

impl AppTuiError {
    pub fn code(&self) -> u16 {
        match self {
            AppTuiError::DetermineTerminalSize(_) => 5000,
            AppTuiError::InitializeTerminal(_) => 5001,
            AppTuiError::RestoreTerminal(_) => 5002,
            AppTuiError::SendMsg(_) => 5003,
            AppTuiError::DrawFrame(_) => 5004,
            AppTuiError::PollForEvents(_) => 5005,
            AppTuiError::ReadEvent(_) => 5006,
        }
    }
}

struct AppTui {
    pub(super) terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(super) event_tx: Sender<Message>,
    pub(super) event_rx: Receiver<Message>,
    pub(super) model: Model,
    pub(super) initial_commands: Vec<Command>,
}

impl AppTui {
    pub fn new(pool: &Pool<Sqlite>, context: TuiContext) -> Result<Self, AppTuiError> {
        let terminal = ratatui::try_init().map_err(AppTuiError::InitializeTerminal)?;
        let (event_tx, event_rx) = mpsc::channel(10);
        let mut initial_commands = Vec::new();

        let (width, height) =
            ratatui::crossterm::terminal::size().map_err(AppTuiError::DetermineTerminalSize)?;

        let terminal_dimensions = TerminalDimensions { width, height };

        match &context {
            TuiContext::Initial => {}
            TuiContext::Search(q) => {
                initial_commands.push(Command::SearchBookmarks(q.clone()));
            }
            TuiContext::Tags => {
                initial_commands.push(Command::FetchTags);
            }
        }

        let model = Model::default(pool, context, terminal_dimensions);

        Ok(Self {
            terminal,
            event_tx,
            event_rx,
            model,
            initial_commands,
        })
    }

    pub async fn run(&mut self) -> Result<(), AppTuiError> {
        let _ = self.terminal.clear();

        for cmd in &self.initial_commands {
            handle_command(&self.model.pool, cmd.clone(), self.event_tx.clone()).await;
        }

        // first render
        self.model.render_counter += 1;
        self.terminal
            .draw(|f| view(&mut self.model, f))
            .map_err(AppTuiError::DrawFrame)?;

        loop {
            tokio::select! {
                Some(message) = self.event_rx.recv() => {
                    let cmds = self.update(message).await;

                    if self.model.running_state == RunningState::Done {
                        self.exit().map_err(AppTuiError::RestoreTerminal)?;
                        return Ok(());
                    }

                        self.model.render_counter += 1;
                        self.terminal.draw(|f| view(&mut self.model, f)).map_err(AppTuiError::DrawFrame)?;

                    for cmd in cmds {
                        handle_command(&self.model.pool, cmd, self.event_tx.clone()).await;
                    }
                }

                Ok(ready) = tokio::task::spawn_blocking(|| ratatui::crossterm::event::poll(Duration::from_millis(EVENT_POLL_DURATION_MS))) => {
                    match ready {
                        Ok(true) => {
                            let event = ratatui::crossterm::event::read().map_err(AppTuiError::ReadEvent)?;
                            self.model.event_counter += 1;
                            if let Some(handling_msg) = self.get_event_handling_msg(event) {
                                self.event_tx.try_send(handling_msg)?;
                            }
                        }
                        Ok(false) => continue,
                        Err(e) => {
                                return Err(AppTuiError::PollForEvents(e));
                        }
                    }
                }
            }
        }
    }

    fn get_event_handling_msg(&self, event: Event) -> Option<Message> {
        match event {
            Event::Key(key_event) => match self.model.terminal_too_small {
                true => match key_event.kind {
                    KeyEventKind::Press => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => Some(Message::GoBackOrQuit),
                        _ => None,
                    },
                    _ => None,
                },
                false => match key_event.kind {
                    KeyEventKind::Press => match self.model.active_pane {
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

    pub(super) async fn update(&mut self, msg: Message) -> Vec<Command> {
        let mut cmds = Vec::new();
        match msg {
            Message::GoToNextListItem => self.model.select_next_list_item(),
            Message::GoToPreviousListItem => self.model.select_previous_list_item(),
            Message::OpenInBrowser => {
                if let Some(c) = self.model.get_cmd_to_open_selection_in_browser() {
                    cmds.push(c)
                }
            }
            Message::UrlsOpenedInBrowser(result) => {
                if let UrlsOpenedResult::Failure(e) = result {
                    self.model.user_message = Some(UserMessage::error(&format!(
                        "urls couldn't be opened: {}",
                        e
                    )));
                }
            }
            Message::GoBackOrQuit => self.model.go_back_or_quit(),
            Message::ShowView(view) => {
                if let Some(c) = self.model.show_view(view) {
                    cmds.push(c);
                }
            }
            Message::GoToFirstListItem => self.model.select_first_list_item(),
            Message::GoToLastListItem => self.model.select_last_list_item(),
            Message::SearchFinished(result) => match result {
                Ok(bookmarks) => {
                    if bookmarks.is_empty() {
                        self.model.user_message =
                            Some(UserMessage::info("no bookmarks found for query"));
                        self.model.bookmark_items = BookmarkItems::from(vec![]);
                    } else {
                        let bookmarks_len = bookmarks.len();
                        if let Some(current_index) = self.model.bookmark_items.state.selected() {
                            if current_index < bookmarks_len {
                                self.model.bookmark_items =
                                    BookmarkItems::from((bookmarks, current_index));
                            } else {
                                self.model.bookmark_items =
                                    BookmarkItems::from((bookmarks, bookmarks_len - 1));
                            }
                        } else {
                            self.model.bookmark_items = BookmarkItems::from(bookmarks);
                        }
                    }
                }
                Err(e) => self.model.user_message = Some(UserMessage::error(&format!("{}", e))),
            },
            Message::TagsFetched(result) => match result {
                Ok(t) => {
                    self.model.tag_items = TagItems::from(t);
                    self.model.active_pane = ActivePane::TagsList;
                }
                Err(e) => self.model.user_message = Some(UserMessage::error(&format!("{}", e))),
            },
            Message::SearchInputGotEvent(event) => {
                self.model.search_input.handle_event(&event);
            }
            Message::SubmitSearch => {
                let search_query = self.model.search_input.value();
                match SearchTerms::try_from(search_query) {
                    Ok(search_terms) => {
                        if !search_query.is_empty() {
                            cmds.push(Command::SearchBookmarks(search_terms));
                            if self.model.initial {
                                self.model.initial = false;
                            }
                        }
                        self.model.search_input.reset();
                        self.model.active_pane = ActivePane::List;
                    }
                    Err(e) => self.model.user_message = Some(UserMessage::error(&format!("{}", e))),
                }
            }
            Message::TerminalResize(width, height) => {
                self.model.terminal_dimensions = TerminalDimensions { width, height };
                self.model.terminal_too_small =
                    !(width >= MIN_TERMINAL_WIDTH && height >= MIN_TERMINAL_HEIGHT);
            }
            Message::ShowBookmarksForTag => {
                if let Some(current_tag_index) = self.model.tag_items.state.selected() {
                    if let Some(selected_tag) = self.model.tag_items.items.get(current_tag_index) {
                        cmds.push(Command::FetchBookmarksForTag(selected_tag.name.to_string()));
                    }
                }
            }
            Message::BookmarksForTagFetched(result) => match result {
                Ok(bookmarks) => {
                    self.model.bookmark_items = BookmarkItems::from(bookmarks);
                    self.model.active_pane = ActivePane::List;
                }
                Err(e) => self.model.user_message = Some(UserMessage::error(&format!("{}", e))),
            },
            Message::CopyURIToClipboard => {
                if let Some(uri) = self.model.get_uri_under_cursor() {
                    cmds.push(Command::CopyContentToClipboard(uri));
                }
            }
            Message::CopyURIsToClipboard => {
                let uris = self
                    .model
                    .bookmark_items
                    .items
                    .iter()
                    .map(|bi| bi.bookmark.uri.as_str())
                    .collect::<Vec<_>>();

                if !uris.is_empty() {
                    cmds.push(Command::CopyContentToClipboard(uris.join("\n")));
                }
            }
            Message::ContentCopiedToClipboard(result) => {
                if let Err(error) = result {
                    self.model.user_message = Some(UserMessage::error(&format!(
                        "couldn't copy uri to clipboard: {}",
                        error
                    )));
                } else {
                    self.model.user_message =
                        Some(UserMessage::info("copied!").with_frames_left(1));
                }
            }
        }

        if let Some(message) = &mut self.model.user_message {
            let clear = if message.frames_left == 0 {
                true
            } else {
                message.frames_left -= 1;
                false
            };

            if clear {
                self.model.user_message = None;
            }
        }

        cmds
    }

    fn exit(&mut self) -> Result<(), IOError> {
        ratatui::try_restore()
    }
}
