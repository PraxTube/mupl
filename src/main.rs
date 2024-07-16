mod load;
mod playlist;
mod song;
mod ui;
mod utils;

use std::error::Error;
use std::sync::mpsc;

use song::{stream_song, ActionData};
use ui::terminal;

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<ActionData>();
    let _streaming_thread = stream_song(rx);
    terminal::setup(tx)
}
