use super::commands::Command;
use super::common::*;
use super::handle::handle_command;
use super::message::*;
use super::model::*;
use super::view::view;
use crate::domain::SavedBookmark;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::Terminal;
use sqlx::{Pool, Sqlite};
use std::io::Error as IOError;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tui_input::backend::crossterm::EventHandler;

const EVENT_POLL_DURATION_MS: u64 = 16;

#[derive(thiserror::Error, Debug)]
pub enum AppTuiError {
    #[error("couldn't initialize bmm's TUI: {0}")]
    InitializeTerminal(IOError),
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

pub async fn run_tui(
    pool: &Pool<Sqlite>,
    bookmarks: Vec<SavedBookmark>,
) -> Result<(), AppTuiError> {
    let mut tui = AppTui::new(pool, bookmarks)?;
    tui.run().await?;

    Ok(())
}

impl AppTuiError {
    pub fn code(&self) -> u16 {
        match self {
            AppTuiError::InitializeTerminal(_) => 5000,
            AppTuiError::RestoreTerminal(_) => 5001,
            AppTuiError::SendMsg(_) => 5002,
            AppTuiError::DrawFrame(_) => 5003,
            AppTuiError::PollForEvents(_) => 5004,
            AppTuiError::ReadEvent(_) => 5005,
        }
    }
}

const CLEAR_USER_MESSAGE_LOOP_INTERVAL_SECS: u64 = 10;

struct AppTui {
    pub(super) terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(super) event_tx: Sender<Message>,
    pub(super) event_rx: Receiver<Message>,
    pub(super) model: Model,
}

impl AppTui {
    pub fn new(pool: &Pool<Sqlite>, bookmarks: Vec<SavedBookmark>) -> Result<Self, AppTuiError> {
        let terminal = ratatui::try_init().map_err(AppTuiError::InitializeTerminal)?;
        let (event_tx, event_rx) = mpsc::channel(10);
        let model = Model::default(pool, bookmarks);
        Ok(Self {
            terminal,
            event_tx,
            event_rx,
            model,
        })
    }

    pub async fn run(&mut self) -> Result<(), AppTuiError> {
        let message_clear_duration = Duration::from_secs(CLEAR_USER_MESSAGE_LOOP_INTERVAL_SECS);
        let mut message_clear_interval = tokio::time::interval(message_clear_duration);
        let _ = self.terminal.clear();

        // first render
        self.model.render_counter += 1;
        self.terminal
            .draw(|f| view(&mut self.model, f))
            .map_err(AppTuiError::DrawFrame)?;

        loop {
            tokio::select! {
                _instant = message_clear_interval.tick() => {
                    if self.model.user_message.is_some() {
                        _ = self.event_tx.try_send(Message::ClearUserMsg);
                    }
                }

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
                true => None,
                false => match key_event.kind {
                    KeyEventKind::Press => match self.model.active_pane {
                        ActivePane::List => match key_event.code {
                            KeyCode::Char('j') | KeyCode::Down => Some(Message::GoToNextItem),
                            KeyCode::Char('k') | KeyCode::Up => Some(Message::GoToPreviousItem),
                            KeyCode::Char('g') => Some(Message::GoToFirstItem),
                            KeyCode::Char('G') => Some(Message::GoToLastItem),
                            KeyCode::Char('o') => Some(Message::OpenInBrowser),
                            KeyCode::Char('s') => Some(Message::ShowView(ActivePane::SearchInput)),
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
                            KeyCode::Down => Some(Message::GoToNextItem),
                            KeyCode::Up => Some(Message::GoToPreviousItem),
                            _ => Some(Message::SearchInputGotEvent(event)),
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
            Message::GoToNextItem => self.model.select_next(),
            Message::GoToPreviousItem => self.model.select_previous(),
            Message::OpenInBrowser => {
                if let Some(c) = self.model.get_cmd_to_open_selection_in_browser() {
                    cmds.push(c)
                }
            }
            Message::UrlsOpenedInBrowser(result) => {
                let message = match result {
                    UrlsOpenedResult::Success => UserMessage::info("urls opened!"),
                    UrlsOpenedResult::Failure(e) => {
                        UserMessage::error(&format!("urls couldn't be opened: {}", e))
                    }
                };
                self.model.user_message = Some(message);
            }
            Message::GoBackOrQuit => self.model.go_back_or_quit(),
            Message::ShowView(view) => self.model.show_view(view),
            Message::GoToFirstItem => self.model.select_first(),
            Message::GoToLastItem => self.model.select_last(),
            Message::SearchFinished(result) => match result {
                Ok(bookmarks) => {
                    if bookmarks.is_empty() {
                        self.model.user_message =
                            Some(UserMessage::info("no bookmarks found for query"));
                        self.model.bookmark_items = BookmarkItems::from(vec![]);
                    } else {
                        let message = if bookmarks.len() == 1 {
                            "1 bookmark found".to_string()
                        } else {
                            format!("{} bookmarks found", bookmarks.len())
                        };
                        self.model.user_message = Some(UserMessage::info(&message));
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
            Message::SearchInputGotEvent(event) => {
                self.model.search_input.handle_event(&event);
                //let search_query = self.model.search_input.value();
                //if !search_query.is_empty() && search_query.len() > 1 {
                //    cmds.push(Command::SearchBookmarks(search_query.to_string()));
                //    if self.model.initial {
                //        self.model.initial = false;
                //    }
                //}
            }
            Message::SubmitSearch => {
                let search_query = self.model.search_input.value();
                if !search_query.is_empty() {
                    cmds.push(Command::SearchBookmarks(search_query.to_string()));
                    if self.model.initial {
                        self.model.initial = false;
                    }
                }
                self.model.search_input.reset();
                self.model.active_pane = ActivePane::List;
            }
            Message::ClearUserMsg => {
                let now = Instant::now();
                let reset_message = match &self.model.user_message {
                    Some(message) => match message {
                        UserMessage::Info(_, instant) => {
                            now.saturating_duration_since(instant.to_owned()).as_secs()
                                > CLEAR_USER_MESSAGE_LOOP_INTERVAL_SECS
                        }
                        UserMessage::Error(_, instant) => {
                            now.saturating_duration_since(instant.to_owned()).as_secs()
                                > CLEAR_USER_MESSAGE_LOOP_INTERVAL_SECS
                        }
                    },
                    None => false,
                };

                if reset_message {
                    self.model.user_message = None;
                }
            }
            Message::TerminalResize(w, h) => {
                self.model.terminal_dimensions = TerminalDimensions::Known(w, h);
                self.model.terminal_too_small = w < MIN_TERMINAL_WIDTH || h < MIN_TERMINAL_HEIGHT;
            }
        }

        cmds
    }

    fn exit(&mut self) -> Result<(), IOError> {
        ratatui::try_restore()
    }
}
