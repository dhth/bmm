use super::super::model::RunningState;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui};
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;

#[test]
fn terminal_too_small_view_is_shown_when_width_is_too_small() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(80, 24);

    // WHEN
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn terminal_too_small_view_is_shown_when_height_is_too_small() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 20);

    // WHEN
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn non_exit_keypresses_are_ignored_when_terminal_is_too_small() {
    // GIVEN
    let (_, mut model) = setup_test_tui(80, 20);

    // WHEN
    let result = press_key(&mut model, KeyCode::Char('j'));

    // THEN
    assert!(result.is_none());
}

#[test]
fn pressing_q_quits_when_terminal_is_too_small() {
    // GIVEN
    let (_, mut model) = setup_test_tui(80, 20);

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('q')).expect("q should be handled");

    // THEN
    assert_eq!(model.running_state, RunningState::Done);
}
