use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use serde_json;

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

pub fn song_data() -> Result<serde_json::Value, Box<dyn Error>> {
    let path = data_path()?;
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json_data: serde_json::Value = serde_json::from_str(&contents)?;
    Ok(json_data)
}

pub fn data_keys() -> Result<Vec<String>, Box<dyn Error>> {
    let song_data = song_data()?;
    let keys: Vec<String> = song_data
        .as_object()
        .ok_or("Not a valid dict.")?
        .keys()
        .cloned()
        .collect();
    Ok(keys)
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
    create_default_file(data_path()?)
}
