use super::super::commands::Command;
use super::super::common::ActivePane;
use super::super::message::Message;
use super::super::model::TuiContext;
use super::super::update::update;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui_with_context};
use crate::domain::{SavedBookmark, TagStats};
use crate::persistence::{DBError, SearchTerms};
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;
use sqlx::Error as SqlxError;

#[test]
fn opening_tui_with_tags_displays_available_tags() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Tags);

    // WHEN
    update(&mut model, Message::TagsFetched(Ok(available_tags())));
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_eq!(model.active_pane, ActivePane::TagsList);
    assert_snapshot!(terminal.backend());
}

#[test]
fn pressing_t_from_bookmarks_fetches_tags() {
    // GIVEN
    let search_terms = SearchTerms::try_from("rust").expect("search terms should be valid");
    let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));

    // WHEN
    let commands = press_key(&mut model, KeyCode::Char('t')).expect("t should be handled");

    // THEN
    let [Command::FetchTags] = commands.as_slice() else {
        panic!("opening tags should emit one fetch tags command");
    };
    assert_eq!(model.active_pane, ActivePane::TagsList);
}

#[test]
fn moving_to_the_next_tag_updates_the_selection() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Tags);
    update(&mut model, Message::TagsFetched(Ok(available_tags())));

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('j')).expect("j should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn selecting_a_tag_displays_its_bookmarks() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Tags);
    update(&mut model, Message::TagsFetched(Ok(available_tags())));
    let _ = press_key(&mut model, KeyCode::Char('j')).expect("j should be handled");

    // WHEN
    let commands = press_key(&mut model, KeyCode::Enter).expect("enter should be handled");
    update(
        &mut model,
        Message::BookmarksForTagFetched(Ok(programming_bookmarks())),
    );
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    let [Command::FetchBookmarksForTag(tag)] = commands.as_slice() else {
        panic!("selecting a tag should emit one fetch bookmarks command");
    };
    assert_eq!(tag, "programming");
    assert_eq!(model.active_pane, ActivePane::List);
    assert_snapshot!(terminal.backend());
}

#[test]
fn reopening_tags_does_not_fetch_them_again() {
    // GIVEN
    let search_terms = SearchTerms::try_from("rust").expect("search terms should be valid");
    let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(
        &mut model,
        Message::SearchFinished(Ok(programming_bookmarks())),
    );
    let initial_commands = press_key(&mut model, KeyCode::Char('t')).expect("t should be handled");
    update(&mut model, Message::TagsFetched(Ok(available_tags())));
    let _ = press_key(&mut model, KeyCode::Esc).expect("escape should be handled");
    assert_eq!(model.active_pane, ActivePane::List);

    // WHEN
    let reopening_commands =
        press_key(&mut model, KeyCode::Char('t')).expect("t should be handled");

    // THEN
    let [Command::FetchTags] = initial_commands.as_slice() else {
        panic!("opening tags for the first time should emit one fetch tags command");
    };
    assert!(reopening_commands.is_empty());
    assert_eq!(model.active_pane, ActivePane::TagsList);
    assert_eq!(model.tag_items.items.len(), 5);
}

#[test]
fn failed_tag_fetch_shows_an_error() {
    // GIVEN
    let (mut terminal, mut model) = setup_test_tui_with_context(120, 24, TuiContext::Tags);

    // WHEN
    update(
        &mut model,
        Message::TagsFetched(Err(DBError::CouldntExecuteQuery(
            "fetch tags".to_string(),
            SqlxError::Protocol("database unavailable".to_string()),
        ))),
    );
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_eq!(model.active_pane, ActivePane::TagsList);
    assert!(model.tag_items.items.is_empty());
    assert_snapshot!(terminal.backend());
}

fn available_tags() -> Vec<TagStats> {
    vec![
        TagStats {
            name: "rust".to_string(),
            num_bookmarks: 5,
        },
        TagStats {
            name: "programming".to_string(),
            num_bookmarks: 4,
        },
        TagStats {
            name: "books".to_string(),
            num_bookmarks: 3,
        },
        TagStats {
            name: "reference".to_string(),
            num_bookmarks: 2,
        },
        TagStats {
            name: "tools".to_string(),
            num_bookmarks: 1,
        },
    ]
}

fn programming_bookmarks() -> Vec<SavedBookmark> {
    vec![
        SavedBookmark {
            uri: "https://www.rust-lang.org/".to_string(),
            title: Some("Rust".to_string()),
            tags: Some("rust,programming".to_string()),
        },
        SavedBookmark {
            uri: "https://doc.rust-lang.org/book/".to_string(),
            title: Some("The Rust Programming Language".to_string()),
            tags: Some("rust,programming,books".to_string()),
        },
        SavedBookmark {
            uri: "https://gleam.run/".to_string(),
            title: Some("Gleam".to_string()),
            tags: Some("gleam,programming".to_string()),
        },
    ]
}
