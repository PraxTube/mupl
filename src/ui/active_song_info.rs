use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, LineGauge, Paragraph},
    Frame,
};

use crate::song::SongInfo;
use crate::ui::terminal::App;
use crate::utils::format_time;

pub fn render_active_song_info<B: Backend>(
    f: &mut Frame<B>,
    app: &mut App,
    chunk: Rect,
    song_info: SongInfo,
) {
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

    let paragraph_info = Paragraph::new(format!(
        "\nName: {}\nFile: {}",
        app.song_data[&song_info.name]["name"], app.song_data[&song_info.name]["artist"][0]
    ))
    .alignment(Alignment::Left);
    f.render_widget(paragraph_info, playing_song_chunks[0]);

    let label = format!(
        "{}/{} - ({:.0}%)",
        format_time(app.progress),
        format_time(song_info.duration),
        app.progress as f64 / song_info.duration as f64 * 100.0
    );
    let ratio: f64 = (app.get_progress() as f64 / song_info.duration as f64)
        .max(0.0)
        .min(1.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(ratio)
        .label(label.clone());
    f.render_widget(gauge, playing_song_chunks[1]);

    let gauge = LineGauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(label);
    f.render_widget(gauge, playing_song_chunks[2]);
}
