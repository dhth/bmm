use super::commands::Command;
use super::common::*;
use super::handle::handle_command;
use super::message::{Message, get_event_handling_msg};
use super::model::*;
use super::update::update;
use super::view::view;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use sqlx::{Pool, Sqlite};
use std::io::Error as IOError;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};

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
                    let cmds = update(&mut self.model, message);

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
                            if let Some(handling_msg) = get_event_handling_msg(&self.model, event) {
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

    fn exit(&mut self) -> Result<(), IOError> {
        ratatui::try_restore()
    }
}
