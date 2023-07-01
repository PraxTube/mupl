mod data;
mod load;
mod playlist;
mod song;
mod terminal;

use std::error::Error;
use std::sync::mpsc;

use clap::Parser;

use song::{stream_song, SongInfo};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::new())]
    file: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<SongInfo>();
    data::check_default_files()?;

    let _streaming_thread = stream_song(rx);
    terminal::setup(tx)
}
