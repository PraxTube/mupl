use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use serde_json;
use walkdir::WalkDir;

fn data_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            return Err("Unable to determine user's home directory".into());
        }
    };

    let data_dir = home_dir.join(".config").join("mupl");

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    Ok(data_dir)
}

fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    let path: PathBuf = data_dir()?.join("config.json");
    Ok(path)
}

fn data_path() -> Result<PathBuf, Box<dyn Error>> {
    let path: PathBuf = data_dir()?.join("data.json");
    Ok(path)
}

fn playlist_path() -> Result<PathBuf, Box<dyn Error>> {
    let path: PathBuf = data_dir()?.join("playlist.json");
    Ok(path)
}

fn get_data(path: PathBuf) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json_data: serde_json::Value = serde_json::from_str(&contents)?;
    Ok(json_data)
}

pub fn song_data() -> Result<serde_json::Value, Box<dyn Error>> {
    let path = data_path()?;
    get_data(path)
}

pub fn playlist_data() -> Result<serde_json::Value, Box<dyn Error>> {
    let path = playlist_path()?;
    get_data(path)
}

pub fn config_data() -> Result<serde_json::Value, Box<dyn Error>> {
    let path = config_path()?;
    get_data(path)
}

fn create_default_file(file: PathBuf) -> Result<(), Box<dyn Error>> {
    if file.exists() {
        return Ok(());
    }

    let json_data = r#"
    {
    }
    "#;
    let parsed_data: serde_json::Value = serde_json::from_str(json_data)?;

    let mut file = OpenOptions::new().create(true).write(true).open(file)?;
    let serialized = serde_json::to_string_pretty(&parsed_data)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn check_default_files() -> Result<(), Box<dyn Error>> {
    create_default_file(config_path()?)?;
    create_default_file(data_path()?)?;
    create_default_file(playlist_path()?)
}

pub fn write_playlist_data(data: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::create(playlist_path()?)?;
    serde_json::to_writer_pretty(file, &data)?;
    Ok(())
}

pub fn song_filestem_to_path(filestem: &str) -> Option<PathBuf> {
    fn music_dir() -> String {
        match config_data() {
            Ok(data) => {
                return data["music-folder"].to_string().replace("\"", "");
            }
            Err(err) => panic!("Couldn't read config data {}", err),
        }
    }

    for entry in WalkDir::new(music_dir()).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if filestem
            == PathBuf::from(path)
                .file_stem()
                .expect("Not a valid file")
                .to_str()
                .expect("Can not convert file to str")
        {
            return Some(PathBuf::from(entry.path()));
        }
    }
    None
}
