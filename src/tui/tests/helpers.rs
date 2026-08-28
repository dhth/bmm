use super::super::commands::Command;
use super::super::common::TerminalDimensions;
use super::super::message::get_event_handling_msg;
use super::super::model::{Model, TuiContext};
use super::super::update::update;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub(super) fn setup_test_tui(width: u16, height: u16) -> (Terminal<TestBackend>, Model) {
    let terminal =
        Terminal::new(TestBackend::new(width, height)).expect("terminal should've been created");
    let model = Model::default(
        TuiContext::Initial,
        TerminalDimensions { width, height },
        false,
    );

    (terminal, model)
}

pub(super) fn press_key(model: &mut Model, key: KeyCode) -> Option<Vec<Command>> {
    press_key_with_modifiers(model, key, KeyModifiers::NONE)
}

pub(super) fn press_key_with_modifiers(
    model: &mut Model,
    key: KeyCode,
    modifiers: KeyModifiers,
) -> Option<Vec<Command>> {
    let event = Event::Key(KeyEvent::new(key, modifiers));
    let message = get_event_handling_msg(model, event)?;

    Some(update(model, message))
}
