use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, LineGauge, Paragraph},
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
    let ratio: f64 = (app.progress as f64 / song_info.duration as f64)
        .max(0.0)
        .min(1.0);
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
        Paragraph::new(format!("\nNAME OF FILE HERE, TODO",)).alignment(Alignment::Left);
    f.render_widget(paragraph_info, playing_song_chunks[0]);

    f.render_widget(
        progress_gauge(app, song_info.clone()),
        playing_song_chunks[1],
    );

    f.render_widget(volume_gauge(app), playing_song_chunks[2]);
}
