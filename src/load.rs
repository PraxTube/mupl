use walkdir::WalkDir;

fn music_dir() -> String {
    "/home/rancic/Music/".to_string()
}

pub fn load_music_files() -> Vec<std::path::PathBuf> {
    let mut music_files = Vec::new();

    // Recursively iterate through the directory and its subfolders
    for entry in WalkDir::new(music_dir()).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "wav") {
            music_files.push(path.to_path_buf());
        }
    }

    music_files
}
