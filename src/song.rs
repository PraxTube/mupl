use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use lofty::prelude::*;
use lofty::probe::Probe;
use rodio::{OutputStream, Sink};

pub enum DataType {
    Int(i32),
    SongInfo(SongInfo),
    Null,
}
pub enum Action {
    TogglePause,
    Volume,
    AddSong,
}

pub struct SetupAudio;
pub struct ActionData {
    pub action: Action,
    pub data: DataType,
}

#[derive(Clone)]
pub struct SongInfo {
    pub name: String,
    pub duration: u32,
    pub file: String,
}

impl SongInfo {
    pub fn new(song_file: PathBuf) -> SongInfo {
        let tagged_file = Probe::open(&song_file)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");
        let duration = tagged_file.properties().duration().as_secs() as u32;

        SongInfo {
            name: song_file.file_name().unwrap().to_str().unwrap().to_owned(),
            duration,
            file: song_file.to_str().unwrap().to_string(),
        }
    }
}

fn add_song_to_sink(song_info: SongInfo, sink: &Sink) {
    sink.stop();
    let file = std::fs::File::open(song_info.file).unwrap();
    let source = rodio::Decoder::new(file).unwrap();
    sink.append(source);
}

fn toggle_pause(sink: &Sink) {
    if sink.is_paused() {
        sink.play();
    } else {
        sink.pause();
    }
}

fn change_volume(volume: i32, sink: &Sink) {
    sink.set_volume(volume as f32 * 0.01);
}

fn match_action(action_data: ActionData, sink: &Sink) {
    match action_data.action {
        Action::AddSong => {
            if let DataType::SongInfo(data) = action_data.data {
                add_song_to_sink(data, sink);
            }
        }
        Action::TogglePause => {
            toggle_pause(sink);
        }
        Action::Volume => {
            if let DataType::Int(volume) = action_data.data {
                change_volume(volume, sink)
            }
        }
    }
}

fn play_song(tx: Sender<SetupAudio>, rx: Receiver<ActionData>) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    tx.send(SetupAudio).unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    loop {
        match rx.recv() {
            Ok(action_data) => match_action(action_data, &sink),
            Err(_) => break,
        };
    }
}

pub fn stream_song(tx: Sender<SetupAudio>, rx: Receiver<ActionData>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("song-streaming".into())
        .spawn(move || play_song(tx, rx))
        .unwrap()
}
