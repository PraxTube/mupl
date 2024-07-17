use std::{env::current_dir, path::PathBuf};

use walkdir::WalkDir;

const MUSIC_FILE_EXTENSIONS: [&str; 3] = ["wav", "ogg", "mp3"];

pub fn load_music_files(path: &PathBuf) -> Vec<std::path::PathBuf> {
    let path = if path.is_relative() {
        &current_dir().unwrap().join(path)
    } else {
        path
    };

    let mut music_files = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path.extension().map_or(false, |ext| {
                MUSIC_FILE_EXTENSIONS.contains(&ext.to_str().unwrap_or_default())
            })
        {
            music_files.push(path.to_path_buf());
        }
    }

    music_files
}
