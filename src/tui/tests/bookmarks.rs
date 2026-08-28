use super::super::message::Message;
use super::super::model::TuiContext;
use super::super::update::update;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui_with_context};
use crate::domain::SavedBookmark;
use crate::persistence::SearchTerms;
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;

#[test]
fn bookmark_details_show_missing_values() {
    // GIVEN
    let search_terms = SearchTerms::try_from("example").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(
        &mut model,
        Message::SearchFinished(Ok(vec![SavedBookmark {
            uri: "https://example.com/".to_string(),
            title: None,
            tags: None,
        }])),
    );

    // WHEN
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn moving_to_the_next_bookmark_updates_the_selection() {
    // GIVEN
    let search_terms = SearchTerms::try_from("example").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks(2))));

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('j')).expect("j should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_snapshot!(terminal.backend());
}

#[test]
fn jumping_between_first_and_last_bookmarks_works() {
    // GIVEN
    let search_terms = SearchTerms::try_from("example").expect("search terms should be valid");
    let (mut terminal, mut model) =
        setup_test_tui_with_context(96, 24, TuiContext::Search(search_terms));
    update(&mut model, Message::SearchFinished(Ok(bookmarks(20))));

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('G')).expect("G should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_eq!(model.bookmark_items.state.selected(), Some(19));
    assert_snapshot!(terminal.backend());

    // WHEN
    let _ = press_key(&mut model, KeyCode::Char('g')).expect("g should be handled");
    terminal
        .draw(|frame| view(&mut model, frame))
        .expect("frame should've been drawn");

    // THEN
    assert_eq!(model.bookmark_items.state.selected(), Some(0));
    assert_snapshot!(terminal.backend());
}

fn bookmarks(count: usize) -> Vec<SavedBookmark> {
    (1..=count)
        .map(|index| SavedBookmark {
            uri: format!("https://example.com/{index:02}"),
            title: Some(format!("Bookmark {index:02}")),
            tags: Some("example".to_string()),
        })
        .collect()
}
