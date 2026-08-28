use super::super::common::ActivePane;
use super::super::message::Message;
use super::super::model::{RunningState, TuiContext};
use super::super::update::update;
use super::helpers::{
    press_key, press_key_with_modifiers, setup_test_tui, setup_test_tui_with_context,
};
use crate::domain::{SavedBookmark, TagStats};
use crate::persistence::SearchTerms;
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

#[test]
fn q_and_escape_go_back_from_tags_to_bookmarks() {
    for key in [KeyCode::Char('q'), KeyCode::Esc] {
        // GIVEN
        let search_terms = SearchTerms::try_from("rust").expect("search terms should be valid");
        let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
        update(&mut model, Message::SearchFinished(Ok(bookmarks())));
        let _ = press_key(&mut model, KeyCode::Char('t')).expect("t should be handled");
        update(
            &mut model,
            Message::TagsFetched(Ok(vec![TagStats {
                name: "rust".to_string(),
                num_bookmarks: 1,
            }])),
        );

        // WHEN
        let _ = press_key(&mut model, key).expect("back key should be handled");

        // THEN
        assert_eq!(model.active_pane, ActivePane::List);
        assert_eq!(model.bookmark_items.items.len(), 1);
        assert_eq!(model.running_state, RunningState::Running);
    }
}

#[test]
fn q_and_escape_quit_when_tui_was_opened_with_tags() {
    for key in [KeyCode::Char('q'), KeyCode::Esc] {
        // GIVEN
        let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Tags);
        update(
            &mut model,
            Message::TagsFetched(Ok(vec![TagStats {
                name: "rust".to_string(),
                num_bookmarks: 1,
            }])),
        );

        // WHEN
        let _ = press_key(&mut model, key).expect("quit key should be handled");

        // THEN
        assert_eq!(model.running_state, RunningState::Done);
    }
}

#[test]
fn q_and_escape_quit_from_bookmarks() {
    for key in [KeyCode::Char('q'), KeyCode::Esc] {
        // GIVEN
        let search_terms = SearchTerms::try_from("rust").expect("search terms should be valid");
        let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
        update(&mut model, Message::SearchFinished(Ok(bookmarks())));

        // WHEN
        let _ = press_key(&mut model, key).expect("quit key should be handled");

        // THEN
        assert_eq!(model.running_state, RunningState::Done);
    }
}

fn bookmarks() -> Vec<SavedBookmark> {
    vec![SavedBookmark {
        uri: "https://www.rust-lang.org/".to_string(),
        title: Some("Rust".to_string()),
        tags: Some("rust,programming".to_string()),
    }]
}
