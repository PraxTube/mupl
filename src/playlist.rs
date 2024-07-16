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

        PlaylistInfo {
            playlist: playlist_name.to_string(),
            songs: Vec::new(),
            index: 0,
            stateful_songs: StatefulList::with_items(Vec::new()),
        }
    }
}
