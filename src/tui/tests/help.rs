use super::super::common::ActivePane;
use super::super::model::TuiContext;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui_with_context};
use crate::persistence::SearchTerms;
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;

#[test]
fn opening_help_displays_keybindings() {
    // GIVEN
    let search_terms = SearchTerms::try_from("rust").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(96, 30, TuiContext::Search(search_terms));

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('?')).expect("? should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_eq!(model.active_pane, ActivePane::Help);
    assert_snapshot!(terminal.backend());
}
