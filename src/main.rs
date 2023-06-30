mod data;
mod load;
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

fn input_file() -> String {
    let args = Args::parse();

    if args.file != "" {
        return args.file.clone();
    } else {
        return "dummy.wav".to_string();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<SongInfo>();
    let songs = load::load_music_files("/home/rancic/recordings/");
    data::check_default_files()?;
    let song_data = data::song_data()?;

    let file = input_file();
    let song_info = SongInfo::new(&file);
    let _streaming_thread = stream_song(file, rx);

    terminal::setup(song_info, tx)
}
