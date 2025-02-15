#![allow(unused)]
use super::model::*;
use super::update::*;
use super::view::*;
use crate::domain::SavedBookmark;

pub fn run(bookmarks: Vec<SavedBookmark>) {
    let mut terminal = ratatui::init();
    terminal.clear().unwrap();

    let mut model = Model::default(bookmarks);

    while model.running_state != RunningState::Done {
        terminal.draw(|f| view(&mut model, f)).unwrap();
        let mut current_msg = handle_event(&model);

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

    ratatui::try_restore().unwrap()
}
