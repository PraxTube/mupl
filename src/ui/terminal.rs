use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use crate::data;
use crate::load;
use crate::playlist;
use crate::song;
use crate::ui;

use crate::ui::active_song_info::render_active_song_info;
use crate::ui::song::render_song_list;
use crate::ui::utils::StatefulList;

#[derive(PartialEq)]
enum Controller {
    Main,
    Playlist,
}

pub struct App {
    pub finder_data: crate::ui::fuzzy_finder::Data,
    pub items: StatefulList<song::SongInfo>,

    pub progress: u32,
    pub song_info: Option<song::SongInfo>,
    songs: Vec<std::path::PathBuf>,
    pub song_data: serde_json::Value,

    controller: Controller,
    tx: Sender<song::ActionData>,

    pause: bool,
    quit: bool,
}

impl App {
    pub fn new(
        _songs: Vec<std::path::PathBuf>,
        _song_data: serde_json::Value,
        _tx: Sender<song::ActionData>,
    ) -> App {
        let mut _items: Vec<song::SongInfo> = Vec::new();
        for song in &_songs {
            _items.push(song::SongInfo::new(song));
        }
        App {
            finder_data: crate::ui::fuzzy_finder::Data::new(),
            items: StatefulList::with_items(_items),

            progress: 0,
            song_info: None,
            songs: _songs,
            song_data: _song_data,

            controller: Controller::Main,
            tx: _tx,

            pause: false,
            quit: false,
        }
    }

    pub fn get_progress(&self) -> u32 {
        self.progress
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
                    self.tx.send(song::ActionData {
                        action: song::Action::AddSong,
                        data: song::DataType::SongInfo(self.song_info.clone().unwrap()),
                    });
                }
            }
            None => {}
        }
    }

    fn change_playing_song(&mut self) {
        if self.items.state.selected() == None {
            return ();
        }

        let new_song_info = self.items.items[self.items.state.selected().unwrap()].clone();

        self.progress = 0;
        self.song_info = Some(new_song_info.clone());
        self.tx.send(song::ActionData {
            action: song::Action::AddSong,
            data: song::DataType::SongInfo(new_song_info),
        });
    }

    fn add_to_playlist(&mut self) {
        if self.items.state.selected() != None {
            self.finder_data
                .reset(playlist::playlist_names(), playlist::add_song_to_playlist);
            self.controller = Controller::Playlist;
        }
    }

    pub fn toggle_pause_song(&mut self) {
        if self.song_info.is_none() {
            return;
        }

        self.pause = !self.pause;

        self.tx.send(song::ActionData {
            action: song::Action::TogglePause,
            data: song::DataType::Null,
        });
    }

    pub fn main_controller(&mut self) {
        self.controller = Controller::Main;
    }

    pub fn selected_song(&mut self) -> Option<song::SongInfo> {
        match self.items.state.selected() {
            Some(index) => {
                return Some(self.items.items[index].clone());
            }
            None => return None,
        }
    }
}

pub fn setup(tx: Sender<song::ActionData>) -> Result<(), Box<dyn Error>> {
    let songs = load::load_music_files("/home/rancic/music/");
    let song_data = data::song_data()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let app = App::new(songs, song_data, tx);
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

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
        match app.controller {
            Controller::Main => main_controller::<B>(&mut app, tick_rate, &mut last_tick),
            Controller::Playlist => {
                ui::playlist::controller::<B>(&mut app, tick_rate, &mut last_tick)
            }
        }?;
    }
}

fn main_controller<B: Backend>(
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
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
                KeyCode::Char('p') => app.add_to_playlist(),
                KeyCode::Enter => app.change_playing_song(),
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    render_song_list(f, app, chunks[0]);

    let block = Block::default().title("Playing Song").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    match &app.song_info {
        Some(info) => render_active_song_info(f, app, chunks[1], info.clone()),
        None => {}
    }

    if app.controller == Controller::Playlist {
        ui::playlist::render_popup(f, app);
    }
}
