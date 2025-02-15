#![allow(unused)]

use super::common::*;
use super::model::Model;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListDirection, ListItem, Padding, Paragraph},
    Frame,
};

const HELP_CONTENTS: &str = include_str!("static/help.txt");

pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    match model.view {
        View::List => render_list_view(model, frame),
        View::Help => render_help_view(frame),
    }
}

fn render_list_view(model: &mut Model, frame: &mut Frame) {
    let items: Vec<ListItem> = model
        .bookmark_items
        .items
        .iter()
        .map(ListItem::from)
        .collect();

    let title_style = Style::new().bold().bg(PRIMARY_COLOR).fg(FG_COLOR);
    let list = List::new(items)
        .block(
            Block::bordered()
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .title("bookmarks")
                //.title_style(title_style)
                .padding(Padding::new(1, 0, 1, 1)),
        )
        .style(Style::new().white())
        .repeat_highlight_symbol(true)
        .highlight_style(Style::new().fg(PRIMARY_COLOR))
        .direction(ListDirection::TopToBottom);

    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![
            Constraint::Min(2),
            Constraint::Max(7),
            Constraint::Max(1),
        ])
        .split(frame.area());

    frame.render_stateful_widget(&list, layout[0], &mut model.bookmark_items.state);

    let maybe_selected = model.bookmark_items.state.selected();
    if let Some(selected) = maybe_selected {
        let maybe_bookmark_item = model.bookmark_items.items.get(selected);
        if let Some(bookmark_item) = maybe_bookmark_item {
            let details = format!(
                r#"URI  : {}
Title: {}
Tags : {}
"#,
                bookmark_item.bookmark.uri,
                bookmark_item
                    .bookmark
                    .title
                    .as_deref()
                    .unwrap_or("<NOT SET>"),
                bookmark_item
                    .bookmark
                    .tags
                    .as_deref()
                    .unwrap_or("<NOT SET>")
            );
            let details = Paragraph::new(details)
                .block(
                    Block::bordered()
                        .border_style(Style::default().fg(SECONDARY_COLOR))
                        //.title_style(Style::new().bold().bg(SECONDARY_COLOR).fg(FG_COLOR))
                        .title("details")
                        .padding(Padding::new(1, 0, 1, 0)),
                )
                .style(Style::new().white().on_black())
                .alignment(Alignment::Left);
            //.wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(&details, layout[1]);
        };
    }

    let status_bar_text = vec![Line::from(vec![
        Span::styled(TITLE, Style::new().bold().bg(PRIMARY_COLOR).fg(FG_COLOR)),
        Span::raw(" status bar"),
        ".".into(),
    ])];

    let status_bar = Paragraph::new(status_bar_text).block(Block::default());

    frame.render_widget(&status_bar, layout[2]);
}

fn render_help_view(frame: &mut Frame) {
    let title_style = Style::new().bold().bg(PRIMARY_COLOR).fg(FG_COLOR);

    let block = Block::default()
        .title(" help ")
        .padding(Padding::new(2, 0, 1, 0))
        .title_style(title_style);

    let lines: Vec<Line<'_>> = HELP_CONTENTS.lines().map(Line::from).collect();

    let p = Paragraph::new(lines)
        .block(block)
        .style(Style::new().white())
        .alignment(Alignment::Left);

    frame.render_widget(p, frame.area())
}
