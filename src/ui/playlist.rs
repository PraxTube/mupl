use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Frame};

use crate::data::playlist_data;
use crate::ui::fuzzy_finder;
use crate::ui::terminal::App;

pub fn playlist_names() -> Vec<String> {
    let data = playlist_data();
    if let Err(_) = data {
        return Vec::new();
    }

    let keys: Vec<String> = data
        .unwrap()
        .as_object()
        .expect("not a valid json file")
        .keys()
        .cloned()
        .collect();
    keys
}

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    fuzzy_finder::render_popup(f, app, "Song PlayList");
}

pub fn controller<B: Backend>(
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
    fuzzy_finder::controller::<B>(app, tick_rate, last_tick)
}
