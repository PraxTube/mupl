mod load;
mod song;
mod ui;
mod utils;

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::{self};

use song::{stream_song, SetupAudio, SongAction};
use ui::terminal;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to music folder
    #[arg(required = true)]
    path: String,

    /// Whether to shuffle the songs or not
    #[arg(short, long, default_value_t = false)]
    shuffle: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let path = PathBuf::from_str(&args.path).unwrap();
    assert!(path.is_dir(), "Given path is not a dir, {:?}", path);

    let (tx, rx) = mpsc::channel::<SongAction>();
    let (tx_setup_audio, rx_setup_audio) = mpsc::channel::<SetupAudio>();
    let _streaming_thread = stream_song(tx_setup_audio, rx);

    if let Err(err) = rx_setup_audio.recv() {
        println!("An error occured during audio setup!\nAborting... {err}");
        return Err(Box::new(err));
    };

    terminal::setup(tx, path)
}
