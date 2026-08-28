use super::super::commands::Command;
use super::super::common::ActivePane;
use super::super::message::Message;
use super::super::model::{Model, RunningState};
use super::super::update::update;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui};
use crate::domain::SavedBookmark;
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;

#[test]
fn initial_view_shows_search_input() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 24);

    // WHEN
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn typing_a_query_updates_search_input() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 24);

    // WHEN
    type_search_query(&mut model, "rust");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_eq!(model.search_input.value(), "rust");
    assert_snapshot!(terminal.backend());
}

#[test]
fn submitting_a_search_displays_matching_bookmarks() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 24);
    type_search_query(&mut model, "rust");

    // WHEN
    let commands = press_key(&mut model, KeyCode::Enter).expect("enter should be handled");
    update(
        &mut model,
        Message::SearchFinished(Ok(matching_bookmarks())),
    );
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    let [Command::SearchBookmarks(search_terms)] = commands.as_slice() else {
        panic!("submitting a search should emit one search command");
    };
    assert_eq!(search_terms.iter().as_slice(), ["rust"]);
    assert_snapshot!(terminal.backend());
}

#[test]
fn moving_to_the_next_search_result_updates_the_selection() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 24);
    type_search_query(&mut model, "rust");
    let _ = press_key(&mut model, KeyCode::Enter).expect("enter should be handled");
    update(
        &mut model,
        Message::SearchFinished(Ok(matching_bookmarks())),
    );

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('j')).expect("j should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn empty_search_results_show_a_message() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 24);
    type_search_query(&mut model, "rust");

    // WHEN
    let _ = press_key(&mut model, KeyCode::Enter).expect("enter should be handled");
    update(&mut model, Message::SearchFinished(Ok(vec![])));
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn escape_cancels_search_input() {
    // GIVEN
    let (_, mut model) = setup_test_tui(96, 24);
    type_search_query(&mut model, "rust");
    let _ = press_key(&mut model, KeyCode::Enter).expect("enter should be handled");
    update(
        &mut model,
        Message::SearchFinished(Ok(matching_bookmarks())),
    );
    let _ = press_key(&mut model, KeyCode::Char('s')).expect("s should be handled");
    type_search_query(&mut model, "book");

    // WHEN
    let _ = press_key(&mut model, KeyCode::Esc).expect("escape should be handled");

    // THEN
    assert_eq!(model.active_pane, ActivePane::List);
    assert!(model.search_input.value().is_empty());
    assert_eq!(model.bookmark_items.items.len(), 2);
}

#[test]
fn q_is_entered_as_search_input() {
    // GIVEN
    let (_, mut model) = setup_test_tui(96, 24);

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('q')).expect("q should be handled");

    // THEN
    assert_eq!(model.search_input.value(), "q");
    assert_eq!(model.active_pane, ActivePane::SearchInput);
    assert_eq!(model.running_state, RunningState::Running);
}

#[test]
fn submitting_an_empty_search_shows_an_error() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui(96, 24);

    // WHEN
    let commands = press_key(&mut model, KeyCode::Enter).expect("enter should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert!(commands.is_empty());
    assert_eq!(model.active_pane, ActivePane::SearchInput);
    assert_eq!(model.running_state, RunningState::Running);
    assert_snapshot!(terminal.backend());
}

fn matching_bookmarks() -> Vec<SavedBookmark> {
    vec![
        SavedBookmark {
            uri: "https://www.rust-lang.org/".to_string(),
            title: Some("Rust".to_string()),
            tags: Some("rust,programming".to_string()),
        },
        SavedBookmark {
            uri: "https://doc.rust-lang.org/book/".to_string(),
            title: Some("The Rust Programming Language".to_string()),
            tags: Some("rust,book".to_string()),
        },
    ]
}

fn type_search_query(model: &mut Model, query: &str) {
    for character in query.chars() {
        let _ = press_key(model, KeyCode::Char(character)).expect("character should be handled");
    }
}
