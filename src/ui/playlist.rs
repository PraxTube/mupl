use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::ui::terminal::App;
use crate::{playlist, ui::fuzzy_finder};

pub fn render_popup_add_to_playlist<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    fuzzy_finder::render_popup(f, app, "Add Song to PlayList");
}

pub fn render_popup_play_playlist<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    fuzzy_finder::render_popup(f, app, "Play Playlist");
}

pub fn render_modify_playlist<B: Backend>(
    f: &mut Frame<B>,
    app: &mut App,
    left_rect: Rect,
    righ_rect: Rect,
) {
    let items: Vec<ListItem> = app
        .playlist_info
        .stateful_songs
        .items
        .iter()
        .map(|song| {
            let song_body = Spans::from(Span::styled(song, Style::default()));
            ListItem::new(song_body).style(Style::default())
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Song List"))
        .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_widget(Clear, left_rect);
    f.render_stateful_widget(
        items,
        left_rect,
        &mut app.playlist_info.stateful_songs.state,
    );
}

pub fn controller_add_to_playlist<B: Backend>(
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
    fuzzy_finder::controller::<B>(app, tick_rate, last_tick)
}

pub fn controller_play_playlist<B: Backend>(
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
    fuzzy_finder::controller::<B>(app, tick_rate, last_tick)
}

pub fn controller_modify_playlist<B: Backend>(
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('h') => unselect(app),
                KeyCode::Char('l') => unselect(app),
                KeyCode::Char('j') => next(app),
                KeyCode::Char('k') => previous(app),
                KeyCode::Char('d') => delete_song(app),
                // Modal
                KeyCode::Char('q') => exit(app),
                KeyCode::Esc => exit(app),
                _ => {}
            }
        }
    }
    if last_tick.elapsed() >= tick_rate {
        app.on_tick();
        *last_tick = Instant::now();
    }
    Ok(())
}

fn unselect(app: &mut App) {
    app.playlist_info.stateful_songs.unselect();
}

fn next(app: &mut App) {
    app.playlist_info.stateful_songs.next();
}

fn previous(app: &mut App) {
    app.playlist_info.stateful_songs.previous();
}

fn delete_song(app: &mut App) {}

fn exit(app: &mut App) {
    app.playlist_info = playlist::PlaylistInfo::new("None");
    app.main_controller();
}
