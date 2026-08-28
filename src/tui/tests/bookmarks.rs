use super::super::message::Message;
use super::super::model::TuiContext;
use super::super::update::update;
use super::super::view::view;
use super::helpers::setup_test_tui_with_context;
use crate::domain::SavedBookmark;
use crate::persistence::SearchTerms;
use insta::assert_snapshot;

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
