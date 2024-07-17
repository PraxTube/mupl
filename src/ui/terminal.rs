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
    path::PathBuf,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};

use super::{
    debug::{render_debug_panel, Debugger},
    song::{render_active_song_info, render_song_list},
};
use crate::load::load_music_files;
use crate::song::{SongAction, SongInfo};

pub struct App {
    pub progress: u32,
    pub volume: i32,
    pub songs: Vec<SongInfo>,
    pub debugger: Debugger,

    current_song_index: usize,
    paused: bool,
    quit: bool,
    tx: Sender<SongAction>,
}

impl App {
    pub fn new(tx: Sender<SongAction>, path: &PathBuf) -> App {
        let volume = 50;
        tx.send(SongAction::Volume(volume)).unwrap();
        App {
            progress: 0,
            volume,
            songs: load_music_files(path)
                .iter()
                .map(|f| SongInfo::new(f.to_path_buf()))
                .collect(),
            debugger: Debugger::new(),

            current_song_index: 0,
            paused: false,
            quit: false,
            tx,
        }
    }

    pub fn on_tick(&mut self) {
        if self.paused {
            return;
        }
        if self.current_song_index >= self.songs.len() {
            return;
        }

        self.progress += 1;
        if self.progress > self.songs[self.current_song_index].duration {
            self.progress = 0;
            self.try_play_next_song();
        }
    }

    fn try_get_current_song_info(&self) -> Option<SongInfo> {
        if self.current_song_index >= self.songs.len() {
            return None;
        }
        Some(self.songs[self.current_song_index].clone())
    }

    fn try_play_current_song(&mut self) {
        let song_info = match self.try_get_current_song_info() {
            Some(r) => r,
            None => return,
        };

        self.progress = 0;
        self.tx.send(SongAction::AddSong(song_info)).unwrap();
    }

    fn change_volume(&mut self, amount: i32) {
        self.volume = (self.volume + amount).clamp(0, 100);
        self.tx.send(SongAction::Volume(self.volume)).unwrap();
    }

    fn try_play_next_song(&mut self) {
        self.current_song_index = (self.current_song_index + 1).min(self.songs.len() - 1);
        self.try_play_current_song();
    }

    fn try_play_previous_song(&mut self) {
        if self.current_song_index != 0 {
            self.current_song_index -= 1;
        }
        self.try_play_current_song();
    }

    pub fn toggle_pause_song(&mut self) {
        self.paused = !self.paused;
        self.tx.send(SongAction::TogglePause).unwrap();
    }

    pub fn get_current_song_index(&self) -> usize {
        self.current_song_index.min(self.songs.len() - 1)
    }
}

pub fn setup(tx: Sender<SongAction>, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut app = App::new(tx, &path);
    if app.songs.is_empty() {
        panic!("There are no songs in the given dir, {:?}. Exiting.", path);
    }
    app.try_play_current_song();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
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
        controller(&mut app, tick_rate, &mut last_tick).unwrap();
    }
}

fn controller(app: &mut App, tick_rate: Duration, last_tick: &mut Instant) -> io::Result<()> {
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.quit = true,
                KeyCode::Char('j') => app.try_play_next_song(),
                KeyCode::Char('k') => app.try_play_previous_song(),
                KeyCode::Char('w') => app.change_volume(5),
                KeyCode::Char('b') => app.change_volume(-5),
                KeyCode::Char(' ') => app.toggle_pause_song(),
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
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Min(1)].as_ref())
        .split(chunks[1]);

    let block = Block::default().title("Playing Song").borders(Borders::ALL);
    f.render_widget(block, right_chunks[0]);
    render_debug_panel(f, app, right_chunks[1]);
    render_song_list(f, app, chunks[0]);
    render_active_song_info(
        f,
        app,
        chunks[1],
        app.try_get_current_song_info()
            .unwrap_or(SongInfo::defect()),
    );
}
