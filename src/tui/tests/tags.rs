use super::super::commands::Command;
use super::super::common::ActivePane;
use super::super::message::Message;
use super::super::model::TuiContext;
use super::super::update::update;
use super::super::view::view;
use super::helpers::{press_key, setup_test_tui_with_context};
use crate::domain::TagStats;
use crate::persistence::SearchTerms;
use insta::assert_snapshot;
use ratatui::crossterm::event::KeyCode;

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
