use std::thread;

use rodio::{OutputStream, Sink, Source};

pub struct SongInfo {
    pub name: String,
    pub duration: u32,
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
        }
    }
}

pub fn stream_song(song_file: String) -> thread::JoinHandle<()> {
    let streaming_thread = thread::Builder::new()
        .name("song-streaming".into())
        .spawn(move || {
            let file = std::fs::File::open(song_file).unwrap();
            let source = rodio::Decoder::new(file).unwrap();

            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            /*
            println!(
                "dur: {} minutes",
                source.total_duration().unwrap().as_secs_f32() / 60.0
            );
            */
            sink.append(source);
            sink.sleep_until_end();
        })
        .unwrap();
    streaming_thread
}
