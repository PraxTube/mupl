use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use lofty::prelude::*;
use lofty::probe::Probe;
use rodio::{OutputStream, Sink};

pub enum SongAction {
    AddSong(SongInfo),
    Volume(i32),
    TogglePause,
}

pub struct SetupAudio;

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

fn match_action(action: SongAction, sink: &Sink) {
    match action {
        SongAction::AddSong(song_info) => add_song_to_sink(song_info, sink),
        SongAction::Volume(volume) => change_volume(volume, sink),
        SongAction::TogglePause => toggle_pause(sink),
    }
}

fn play_song(tx: Sender<SetupAudio>, rx: Receiver<SongAction>) {
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

pub fn stream_song(tx: Sender<SetupAudio>, rx: Receiver<SongAction>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("song-streaming".into())
        .spawn(move || play_song(tx, rx))
        .unwrap()
}
