mod load;
mod playlist;
mod song;
mod ui;
mod utils;

use std::error::Error;
use std::sync::mpsc::{self};

use song::{stream_song, ActionData, SetupAudio};
use ui::terminal;

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<ActionData>();
    let (tx_setup_audio, rx_setup_audio) = mpsc::channel::<SetupAudio>();
    let _streaming_thread = stream_song(tx_setup_audio, rx);

    if let Err(err) = rx_setup_audio.recv() {
        println!("An error occured during audio setup!\nAborting... {err}");
        return Err(Box::new(err));
    };

    terminal::setup(tx)
}
