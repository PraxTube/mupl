use serde_json::json;

use crate::data::{self, playlist_data};
use crate::ui::terminal;
use crate::ui::terminal::App;
use crate::ui::utils::StatefulList;

#[derive(Clone)]
pub struct PlaylistInfo {
    pub playlist: String,
    pub songs: Vec<String>,
    pub index: usize,
    pub stateful_songs: StatefulList<String>,
}

impl PlaylistInfo {
    pub fn new(playlist_name: &str) -> PlaylistInfo {
        let playlist_data = playlist_data();
        let mut _songs: Vec<String>;
        match playlist_data {
            Ok(data) => {
                _songs = data[playlist_name]
                    .as_array()
                    .expect("the playlist data is not an array")
                    .iter()
                    .map(|v| v.to_string().replace("\"", ""))
                    .collect()
            }
            Err(err) => panic!("can not access playlist data {}", err),
        }

        PlaylistInfo {
            playlist: playlist_name.to_string(),
            songs: _songs.clone(),
            index: 0,
            stateful_songs: StatefulList::with_items(_songs),
        }
    }
}

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

    app.main_controller();
}

pub fn play_playlist(app: &mut App) {
    app.playlist_info = Some(PlaylistInfo::new(&app.finder_data.output));
    app.main_controller();
}

pub fn modify_playlist(app: &mut App) {
    app.playlist_info = Some(PlaylistInfo::new(&app.finder_data.output));
    app.controller = terminal::Controller::ModifyPlaylist;
}
