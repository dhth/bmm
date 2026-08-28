use super::super::common::ActivePane;
use super::super::model::RunningState;
use super::helpers::{press_key, press_key_with_modifiers, setup_test_tui};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn q_and_escape_go_back_from_help_view() {
    for key in [KeyCode::Char('q'), KeyCode::Esc] {
        // GIVEN
        let (_, mut model) = setup_test_tui(96, 24);
        model.active_pane = ActivePane::List;
        let _ = press_key(&mut model, KeyCode::Char('?')).expect("? should be handled");

        // WHEN
        let _ = press_key(&mut model, key).expect("back key should be handled");

        // THEN
        assert_eq!(model.active_pane, ActivePane::List);
        assert_eq!(model.running_state, RunningState::Running);
    }
}

#[test]
fn ctrl_c_quits_immediately_from_help_view() {
    // GIVEN
    let (_, mut model) = setup_test_tui(96, 24);
    model.active_pane = ActivePane::List;
    let _ = press_key(&mut model, KeyCode::Char('?')).expect("? should be handled");

    // WHEN
    let _ = press_key_with_modifiers(&mut model, KeyCode::Char('c'), KeyModifiers::CONTROL)
        .expect("ctrl+c should be handled");

    // THEN
    assert_eq!(model.running_state, RunningState::Done);
}
