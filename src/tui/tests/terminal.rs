use super::super::common::ActivePane;
use super::super::model::RunningState;
use super::super::view::view;
use super::helpers::{handle_event, press_key, press_key_with_modifiers, setup_test_tui};
use insta::assert_snapshot;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};

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
fn tui_can_quit_when_terminal_is_too_small() {
    let exit_keys = [
        (KeyCode::Char('q'), KeyModifiers::NONE),
        (KeyCode::Esc, KeyModifiers::NONE),
        (KeyCode::Char('c'), KeyModifiers::CONTROL),
    ];

    for (key, modifiers) in exit_keys {
        // GIVEN
        let (_, mut model) = setup_test_tui(80, 20);

        // WHEN
        let _ = press_key_with_modifiers(&mut model, key, modifiers)
            .expect("exit key should be handled");

        // THEN
        assert_eq!(model.running_state, RunningState::Done);
    }
}

#[test]
fn ctrl_c_with_additional_modifiers_does_not_quit() {
    // GIVEN
    let (_, mut model) = setup_test_tui(96, 24);

    // WHEN
    let _ = press_key_with_modifiers(
        &mut model,
        KeyCode::Char('c'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    );

    // THEN
    assert_eq!(model.running_state, RunningState::Running);
}

#[test]
fn tui_recovers_after_terminal_is_resized_back_to_minimum() {
    // GIVEN
    let (_, mut model) = setup_test_tui(96, 24);
    for character in "rust".chars() {
        let _ =
            press_key(&mut model, KeyCode::Char(character)).expect("character should be handled");
    }

    // WHEN
    let _ = handle_event(&mut model, Event::Resize(80, 24)).expect("resize should be handled");

    // THEN
    assert!(model.terminal_too_small());

    // WHEN
    let _ = handle_event(&mut model, Event::Resize(96, 24)).expect("resize should be handled");

    // THEN
    assert!(!model.terminal_too_small());
    assert_eq!(model.active_pane, ActivePane::SearchInput);
    assert_eq!(model.search_input.value(), "rust");
}
