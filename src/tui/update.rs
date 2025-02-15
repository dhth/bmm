#![allow(unused)]

use super::common::View;
use super::message::Message;
use super::model::{Model, RunningState};
use ratatui::crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub(crate) fn handle_event(model: &Model) -> Option<Message> {
    if event::poll(Duration::from_millis(16)).unwrap() {
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == event::KeyEventKind::Press {
                return handle_key(model, key);
            }
        }
    }

    None
}

fn handle_key(model: &Model, key: event::KeyEvent) -> Option<Message> {
    match model.view {
        View::List => match key.code {
            Event::Resize(w, h) => Some(Message::TerminalResize(w, h)),
            KeyCode::Char('j') | KeyCode::Down => Some(Message::GoToNextItem),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::GoToPreviousPreview),
            KeyCode::Char('g') => Some(Message::GoToFirstItem),
            KeyCode::Char('G') => Some(Message::GoToLastItem),
            KeyCode::Char('o') => Some(Message::OpenInBrowser),
            KeyCode::Esc | KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('?') => Some(Message::ShowView(View::Help)),
            _ => None,
        },
        View::Help => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('?') => Some(Message::ShowView(View::Help)),
            _ => None,
        },
    }
}

pub(crate) fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::GoToNextItem => select_next(model),
        Message::GoToPreviousPreview => select_previous(model),
        Message::GoToLastItem => select_last(model),
        Message::GoToFirstItem => select_first(model),
        Message::OpenInBrowser => select_first(model),
        Message::ShowView(v) => show_view(model, v),
        Message::Quit => go_back_or_quit(model),
    }
}

fn select_next(model: &mut Model) -> Option<Message> {
    model.bookmark_items.state.select_next();
    None
}
fn select_previous(model: &mut Model) -> Option<Message> {
    model.bookmark_items.state.select_previous();
    None
}
fn select_first(model: &mut Model) -> Option<Message> {
    model.bookmark_items.state.select_first();
    None
}
fn select_last(model: &mut Model) -> Option<Message> {
    model.bookmark_items.state.select_last();
    None
}

fn show_view(model: &mut Model, view: View) -> Option<Message> {
    model.view = match model.view {
        View::Help => View::List,
        _ => view,
    };
    None
}

fn open_in_browser(urls: Vec<String>) -> Option<Message> {
    None
}

fn go_back_or_quit(model: &mut Model) -> Option<Message> {
    match model.view {
        View::List => model.running_state = RunningState::Done,
        View::Help => model.view = View::List,
    };

    None
}
