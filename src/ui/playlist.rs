use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Frame};

use crate::ui::fuzzy_finder;
use crate::ui::terminal::App;

pub fn render_popup_add_to_playlist<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    fuzzy_finder::render_popup(f, app, "Add Song to PlayList");
}

pub fn render_popup_play_playlist<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    fuzzy_finder::render_popup(f, app, "Play Playlist");
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
