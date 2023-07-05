use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::ui::terminal::App;

pub fn render_song_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let song_body = Spans::from(Span::styled(&i.name, Style::default()));
            ListItem::new(song_body).style(Style::default())
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Song List"))
        .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunk, &mut app.items.state);
}
