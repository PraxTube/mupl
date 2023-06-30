use std::sync::mpsc::Receiver;
use std::thread;

use rodio::{OutputStream, Sink, Source};

#[derive(Clone)]
pub struct SongInfo {
    pub name: String,
    pub duration: u32,
    pub file: String,
}

impl SongInfo {
    pub fn new(song_file: &str) -> SongInfo {
        let file = std::fs::File::open(song_file).unwrap();
        let source = rodio::Decoder::new(file).unwrap();
        let path = std::path::Path::new(song_file);
        let file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        SongInfo {
            name: file_stem,
            duration: source.total_duration().unwrap().as_secs_f32() as u32,
            file: song_file.to_string(),
        }
    }
}

fn add_song_to_sink(song_info: SongInfo, sink: &Sink) {
    sink.stop();
    let file = std::fs::File::open(song_info.file).unwrap();
    let source = rodio::Decoder::new(file).unwrap();

    sink.append(source);
}

fn play_song(rx: Receiver<SongInfo>) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    loop {
        match rx.recv() {
            Ok(song_info) => add_song_to_sink(song_info, &sink),
            Err(_) => break,
        };
    }
}

pub fn stream_song(rx: Receiver<SongInfo>) -> thread::JoinHandle<()> {
    let streaming_thread = thread::Builder::new()
        .name("song-streaming".into())
        .spawn(move || play_song(rx))
        .unwrap();
    streaming_thread
}
