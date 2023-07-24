use crossterm::event::{self, Event, KeyCode};
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

use crate::playlist;
use crate::{data, ui::terminal::App};

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

fn delete_song(app: &mut App) {
    let index = match app.playlist_info.stateful_songs.state.selected() {
        Some(i) => i,
        None => return,
    };

    app.playlist_info.stateful_songs.delete_item(index);
}

fn exit(app: &mut App) {
    let data = data::playlist_data();
    let previous_songs = match data[&app.playlist_info.playlist].as_array() {
        Some(songs) => songs,
        None => panic!("Not a valid playlist name"),
    };

    let dirty_flag = previous_songs.len() != app.playlist_info.stateful_songs.items.len();
    if dirty_flag {
        modify_playlist(app);
    } else {
        neg_modify_playlist(app);
    }
}

fn modify_playlist(app: &mut App) {
    app.confirmation_data.reset(
        "Apply changes to playlist?".to_string(),
        "Confirmation".to_string(),
        pos_modify_playlist,
        neg_modify_playlist,
    );
    app.confirm();
}

pub fn pos_modify_playlist(app: &mut App) {
    playlist::write_modified_playlist(app);
    app.playlist_info = playlist::PlaylistInfo::new("None");
    app.main_controller();
}

pub fn neg_modify_playlist(app: &mut App) {
    app.playlist_info = playlist::PlaylistInfo::new("None");
    app.main_controller();
}

pub fn add_playlist(app: &mut App) {
    let new_playlist = app.text_prompt_data.output.clone();
    let current_data = data::playlist_data();
    if current_data.contains_key(&new_playlist) {
        app.confirmation_data.reset(
            "Playlist already exists, reset it?".to_string(),
            "Reset Playlist".to_string(),
            pos_add_playlist,
            neg_add_playlist,
        );
        app.confirm();
    } else {
        pos_add_playlist(app);
    }
}

pub fn pos_add_playlist(app: &mut App) {
    playlist::add_playlist(app);
    app.main_controller();
}

pub fn neg_add_playlist(app: &mut App) {
    app.main_controller();
}
