use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};

use crate::load;
use crate::playlist;
use crate::song;
use crate::song::SongInfo;

use crate::ui::active_song_info::render_active_song_info;
use crate::ui::debug;
use crate::ui::song::render_song_list;
use crate::ui::utils::StatefulList;

pub struct App {
    pub items: StatefulList<song::SongInfo>,

    pub progress: u32,
    pub volume: i32,
    pub song_info: Option<song::SongInfo>,

    pub playlist_info: playlist::PlaylistInfo,
    pub debugger: debug::Debug,

    tx: Sender<song::ActionData>,

    pause: bool,
    quit: bool,
}

impl App {
    pub fn new(tx: Sender<song::ActionData>) -> App {
        let mut _items: Vec<song::SongInfo> = Vec::new();
        for song in &load::load_music_files() {
            _items.push(SongInfo::new(
                song.file_stem()
                    .expect("Not a valid music file")
                    .to_str()
                    .expect("Can not convert to song file to str")
                    .to_string(),
            ));
        }
        App {
            items: StatefulList::with_items(_items),

            progress: 0,
            volume: 50,
            song_info: None,

            playlist_info: playlist::PlaylistInfo::new("None"),
            debugger: debug::Debug::new(),

            tx,

            pause: false,
            quit: false,
        }
    }

    pub fn on_tick(&mut self) {
        if self.pause {
            return;
        }

        match &self.song_info {
            Some(info) => {
                self.progress += 1;
                if self.progress > info.duration {
                    self.progress = 0;
                    self.change_playing_song();
                }
            }
            None => {}
        }
    }

    fn change_playing_song(&mut self) {
        if self.playlist_info.playlist != "None" {
            self.playback_playlist();
            return;
        }

        if self.items.state.selected().is_none() {
            return;
        }

        let new_song_info = self.items.items[self.items.state.selected().unwrap()].clone();

        self.progress = 0;
        self.song_info = Some(new_song_info.clone());
        if self.pause {
            self.toggle_pause_song();
        }
        self.tx
            .send(song::ActionData {
                action: song::Action::AddSong,
                data: song::DataType::SongInfo(new_song_info),
            })
            .unwrap();
    }

    fn playback_playlist(&mut self) {
        if self.playlist_info.index >= self.playlist_info.songs.len() {
            self.playlist_info = playlist::PlaylistInfo::new("None");
            return;
        }

        let new_song_info =
            SongInfo::new(self.playlist_info.songs[self.playlist_info.index].clone());

        self.progress = 0;
        self.song_info = Some(new_song_info.clone());
        self.tx
            .send(song::ActionData {
                action: song::Action::AddSong,
                data: song::DataType::SongInfo(new_song_info),
            })
            .unwrap();
        self.playlist_info.index += 1;
    }

    fn change_volume(&mut self, amount: i32) {
        self.volume = (self.volume + amount).max(0).min(100);
        self.tx
            .send(song::ActionData {
                action: song::Action::Volume,
                data: song::DataType::Int(self.volume),
            })
            .unwrap();
    }

    pub fn toggle_pause_song(&mut self) {
        if self.song_info.is_none() {
            return;
        }

        self.pause = !self.pause;

        self.tx
            .send(song::ActionData {
                action: song::Action::TogglePause,
                data: song::DataType::Null,
            })
            .unwrap();
    }
}

pub fn setup(tx: Sender<song::ActionData>) -> Result<(), Box<dyn Error>> {
    tx.send(song::ActionData {
        action: song::Action::AddSong,
        data: song::DataType::SongInfo(SongInfo {
            name: "Coool Song".to_string(),
            duration: 3,
            file: "/home/anto/music/rumbling.wav".to_string(),
        }),
    })?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let app = App::new(tx);
    run_app(&mut terminal, app, tick_rate).unwrap();

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        if app.quit {
            return Ok(());
        }

        terminal.draw(|f| ui(f, &mut app))?;
        main_controller(&mut app, tick_rate, &mut last_tick).unwrap();
    }
}

fn main_controller(app: &mut App, tick_rate: Duration, last_tick: &mut Instant) -> io::Result<()> {
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.quit = true,
                KeyCode::Char('h') => app.items.unselect(),
                KeyCode::Char('l') => app.items.unselect(),
                KeyCode::Char('j') => app.items.next(),
                KeyCode::Char('k') => app.items.previous(),
                // Directly on Song
                KeyCode::Char('w') => app.change_volume(5),
                KeyCode::Char('b') => app.change_volume(-5),
                KeyCode::Char(' ') => app.toggle_pause_song(),
                // Misc
                KeyCode::Enter => app.change_playing_song(),
                _ => {}
            }
        }
    }
    if last_tick.elapsed() >= tick_rate {
        app.on_tick();
        *last_tick = Instant::now();
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(main_chunks[0]);
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
        .split(chunks[1]);

    render_song_list(f, app, chunks[0]);

    let block = Block::default().title("Playing Song").borders(Borders::ALL);
    f.render_widget(block, right_chunks[0]);

    debug::render_active_song_info(f, app, right_chunks[1]);

    match &app.song_info {
        Some(info) => render_active_song_info(f, app, chunks[1], info.clone()),
        None => {}
    }
}
