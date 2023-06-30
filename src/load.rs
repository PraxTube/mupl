use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn load_music_files(dir_path: &str) -> Vec<std::path::PathBuf> {
    let mut music_files = Vec::new();

    // Recursively iterate through the directory and its subfolders
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "wav") {
            music_files.push(path.to_path_buf());
        }
    }

    music_files
}
