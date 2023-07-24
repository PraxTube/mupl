use serde_json::json;

use crate::data::{self, playlist_data, write_playlist_data};
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
        if playlist_name == "None" {
            return PlaylistInfo {
                playlist: "None".to_string(),
                songs: vec![],
                index: 0,
                stateful_songs: StatefulList::new(),
            };
        }

        let playlist_data = playlist_data();
        let mut _songs: Vec<String>;
        _songs = playlist_data[playlist_name]
            .as_array()
            .expect("the playlist data is not an array")
            .iter()
            .map(|v| v.to_string().replace("\"", ""))
            .collect();

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

    let keys: Vec<String> = data.keys().cloned().collect();
    keys
}

pub fn add_song_to_playlist(app: &mut App) {
    let mut data = playlist_data();

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

    match data::write_playlist_data(serde_json::Value::Object(data)) {
        Ok(_) => {}
        Err(err) => panic!("There was an error when writing playlist data, {}", err),
    }

    app.main_controller();
}

pub fn play_playlist(app: &mut App) {
    app.playlist_info = PlaylistInfo::new(&app.finder_data.output);
    app.main_controller();
}

pub fn modify_playlist(app: &mut App) {
    app.playlist_info = PlaylistInfo::new(&app.finder_data.output);
    app.controller = terminal::Controller::ModifyPlaylist;
}

pub fn write_modified_playlist(app: &mut App) {
    let mut current_data = playlist_data();
    current_data[&app.playlist_info.playlist] =
        app.playlist_info.stateful_songs.items.clone().into();
    write_playlist_data(serde_json::Value::Object(current_data));
}

pub fn add_playlist(app: &mut App) {
    let mut current_data = playlist_data();
    current_data.insert(app.text_prompt_data.output.clone(), json!([]));
    write_playlist_data(serde_json::Value::Object(current_data));
}
