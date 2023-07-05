use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Frame};

use serde_json::json;

use crate::ui::fuzzy_finder;
use crate::ui::terminal::App;
use crate::{
    data::{self, playlist_data},
    playlist,
};

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

pub fn add_song_to_playlist(app: &mut App) {
    let raw_data = playlist_data();
    if let Err(_) = raw_data {
        panic!("Can not open playlist data");
    }
    let mut data = raw_data.unwrap();

    if data.get(&app.finder_data.output) == None {
        panic!("The given playlist does not exist");
    }

    match app.selected_song() {
        Some(song) => {
            if let Some(playlist_dict) = data.get_mut(&app.finder_data.output) {
                if let Some(playlist) = playlist_dict.as_array_mut() {
                    playlist.push(json!(song.name));
                }
            }
        }
        None => panic!("There is no song selected"),
    }

    match data::write_playlist_data(data) {
        Ok(_) => {}
        Err(err) => panic!("There was an error when writing playlist data, {}", err),
    }
}
