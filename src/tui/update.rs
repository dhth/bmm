use super::commands::Command;
use super::common::*;
use super::message::{Message, UrlsOpenedResult};
use super::model::*;
use crate::persistence::SearchTerms;
use tui_input::backend::crossterm::EventHandler;

pub fn update(model: &mut Model, msg: Message) -> Vec<Command> {
    let mut cmds = Vec::new();
    match msg {
        Message::GoToNextListItem => model.select_next_list_item(),
        Message::GoToPreviousListItem => model.select_previous_list_item(),
        Message::OpenInBrowser => {
            if let Some(c) = model.get_cmd_to_open_selection_in_browser() {
                cmds.push(c)
            }
        }
        Message::UrlsOpenedInBrowser(result) => {
            if let UrlsOpenedResult::Failure(e) = result {
                model.user_message =
                    Some(UserMessage::error(&format!("urls couldn't be opened: {e}")));
            }
        }
        Message::GoBackOrQuit => model.go_back_or_quit(),
        Message::ShowView(view) => {
            if let Some(c) = model.show_view(view) {
                cmds.push(c);
            }
        }
        Message::GoToFirstListItem => model.select_first_list_item(),
        Message::GoToLastListItem => model.select_last_list_item(),
        Message::SearchFinished(result) => match result {
            Ok(bookmarks) => {
                if bookmarks.is_empty() {
                    model.user_message = Some(UserMessage::info("no bookmarks found for query"));
                    model.bookmark_items = BookmarkItems::from(vec![]);
                } else {
                    let bookmarks_len = bookmarks.len();
                    if let Some(current_index) = model.bookmark_items.state.selected() {
                        if current_index < bookmarks_len {
                            model.bookmark_items = BookmarkItems::from((bookmarks, current_index));
                        } else {
                            model.bookmark_items =
                                BookmarkItems::from((bookmarks, bookmarks_len - 1));
                        }
                    } else {
                        model.bookmark_items = BookmarkItems::from(bookmarks);
                    }
                }
            }
            Err(e) => model.user_message = Some(UserMessage::error(&format!("{e}"))),
        },
        Message::TagsFetched(result) => match result {
            Ok(t) => {
                model.tag_items = TagItems::from(t);
                model.active_pane = ActivePane::TagsList;
            }
            Err(e) => model.user_message = Some(UserMessage::error(&format!("{e}"))),
        },
        Message::SearchInputGotEvent(event) => {
            model.search_input.handle_event(&event);
        }
        Message::SubmitSearch => {
            let search_query = model.search_input.value();
            match SearchTerms::try_from(search_query) {
                Ok(search_terms) => {
                    if !search_query.is_empty() {
                        cmds.push(Command::SearchBookmarks(search_terms));
                        if model.initial {
                            model.initial = false;
                        }
                    }
                    model.search_input.reset();
                    model.active_pane = ActivePane::List;
                }
                Err(e) => model.user_message = Some(UserMessage::error(&format!("{e}"))),
            }
        }
        Message::TerminalResize(width, height) => {
            model.terminal_dimensions = TerminalDimensions { width, height };
            model.terminal_too_small =
                !(width >= MIN_TERMINAL_WIDTH && height >= MIN_TERMINAL_HEIGHT);
        }
        Message::ShowBookmarksForTag => {
            if let Some(current_tag_index) = model.tag_items.state.selected() {
                if let Some(selected_tag) = model.tag_items.items.get(current_tag_index) {
                    cmds.push(Command::FetchBookmarksForTag(selected_tag.name.to_string()));
                }
            }
        }
        Message::BookmarksForTagFetched(result) => match result {
            Ok(bookmarks) => {
                model.bookmark_items = BookmarkItems::from(bookmarks);
                model.active_pane = ActivePane::List;
            }
            Err(e) => model.user_message = Some(UserMessage::error(&format!("{e}"))),
        },
        Message::CopyURIToClipboard => {
            if let Some(uri) = model.get_uri_under_cursor() {
                cmds.push(Command::CopyContentToClipboard(uri));
            }
        }
        Message::CopyURIsToClipboard => {
            let uris = model
                .bookmark_items
                .items
                .iter()
                .map(|bi| bi.bookmark.uri.as_str())
                .collect::<Vec<_>>();

            if !uris.is_empty() {
                cmds.push(Command::CopyContentToClipboard(uris.join("\n")));
            }
        }
        Message::ContentCopiedToClipboard(result) => {
            if let Err(error) = result {
                model.user_message = Some(UserMessage::error(&format!(
                    "couldn't copy uri to clipboard: {error}"
                )));
            } else {
                model.user_message = Some(UserMessage::info("copied!").with_frames_left(1));
            }
        }
    }

    if let Some(message) = &mut model.user_message {
        let clear = if message.frames_left == 0 {
            true
        } else {
            message.frames_left -= 1;
            false
        };

        if clear {
            model.user_message = None;
        }
    }

    cmds
}
