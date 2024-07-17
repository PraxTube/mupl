use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, LineGauge, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::song::SongInfo;
use crate::ui::terminal::App;
use crate::utils::format_time;

fn progress_gauge(app: &mut App, song_info: SongInfo) -> LineGauge {
    let label = format!(
        "{}/{} - ({:.0}%)",
        format_time(app.progress),
        format_time(song_info.duration),
        app.progress as f64 / song_info.duration as f64 * 100.0
    );
    let ratio: f64 = (app.progress as f64 / song_info.duration as f64).clamp(0.0, 1.0);
    let gauge = LineGauge::default()
        .block(Block::default().borders(Borders::ALL))
        .filled_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(label);
    gauge
}

fn volume_gauge(app: &mut App) -> LineGauge {
    let ratio = app.volume as f64 * 0.01;
    let label = format!("Volume - ({}%)", app.volume);
    let gauge = LineGauge::default()
        .block(Block::default().borders(Borders::ALL))
        .filled_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(label);
    gauge
}

pub fn render_active_song_info(f: &mut Frame, app: &mut App, chunk: Rect, song_info: SongInfo) {
    let playing_song_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(chunk);

    let paragraph_info =
        Paragraph::new(format!("\nFile: {}", song_info.name)).alignment(Alignment::Left);
    f.render_widget(paragraph_info, playing_song_chunks[0]);

    f.render_widget(
        progress_gauge(app, song_info.clone()),
        playing_song_chunks[1],
    );

    f.render_widget(volume_gauge(app), playing_song_chunks[2]);
}

pub fn render_song_list(f: &mut Frame, app: &mut App, chunk: Rect) {
    let items: Vec<ListItem> = app
        .songs
        .iter()
        .map(|i| {
            let song_body = Line::from(Span::styled(&i.name, Style::default()));
            ListItem::new(song_body).style(Style::default())
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Song List"))
        .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // We can now render the item list
    let mut state = ListState::default();
    state.select_next();
    f.render_stateful_widget(items, chunk, &mut state);
}
