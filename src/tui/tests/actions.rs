use super::super::commands::Command;
use super::super::message::{Message, UrlsOpenedResult};
use super::super::model::TuiContext;
use super::super::update::update;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui_with_context};
use crate::domain::SavedBookmark;
use crate::persistence::SearchTerms;
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;
use std::io::Error as IOError;

#[test]
fn opening_a_bookmark_uses_the_selected_uri() {
    // GIVEN
    let search_terms = SearchTerms::try_from("programming").expect("search terms should be valid");
    let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks())));
    let _ = press_key(&mut model, KeyCode::Char('j')).expect("j should be handled");

    // WHEN
    let commands = press_key(&mut model, KeyCode::Char('o')).expect("o should be handled");

    // THEN
    let [Command::OpenInBrowser(uri)] = commands.as_slice() else {
        panic!("opening a bookmark should emit one open in browser command");
    };
    assert_eq!(uri, "https://gleam.run/");
}

#[test]
fn browser_failure_shows_an_error() {
    // GIVEN
    let search_terms = SearchTerms::try_from("programming").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(120, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks())));

    // WHEN
    update(
        &mut model,
        Message::UrlsOpenedInBrowser(UrlsOpenedResult::Failure(IOError::other(
            "browser unavailable",
        ))),
    );
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn copying_a_bookmark_works() {
    // GIVEN
    let search_terms = SearchTerms::try_from("programming").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks())));
    let _ = press_key(&mut model, KeyCode::Char('j')).expect("j should be handled");

    // WHEN
    let commands = press_key(&mut model, KeyCode::Char('y')).expect("y should be handled");
    update(&mut model, Message::ContentCopiedToClipboard(Ok(())));
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    let [Command::CopyContentToClipboard(uri)] = commands.as_slice() else {
        panic!("copying a bookmark should emit one copy command");
    };
    assert_eq!(uri, "https://gleam.run/");
    assert_snapshot!(terminal.backend());
}

#[test]
fn copying_all_bookmarks_works() {
    // GIVEN
    let search_terms = SearchTerms::try_from("programming").expect("search terms should be valid");
    let (_, mut model) = setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks())));

    // WHEN
    let commands = press_key(&mut model, KeyCode::Char('Y')).expect("Y should be handled");

    // THEN
    let [Command::CopyContentToClipboard(uris)] = commands.as_slice() else {
        panic!("copying all bookmarks should emit one copy command");
    };
    assert_eq!(uris, "https://www.rust-lang.org/\nhttps://gleam.run/");
}

#[test]
fn failed_copy_shows_an_error() {
    // GIVEN
    let search_terms = SearchTerms::try_from("programming").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(120, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks())));

    // WHEN
    update(
        &mut model,
        Message::ContentCopiedToClipboard(Err("clipboard unavailable".to_string())),
    );
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

fn bookmarks() -> Vec<SavedBookmark> {
    vec![
        SavedBookmark {
            uri: "https://www.rust-lang.org/".to_string(),
            title: Some("Rust".to_string()),
            tags: Some("rust,programming".to_string()),
        },
        SavedBookmark {
            uri: "https://gleam.run/".to_string(),
            title: Some("Gleam".to_string()),
            tags: Some("gleam,programming".to_string()),
        },
    ]
}
