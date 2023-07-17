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
use crate::song::SongInfo;
use crate::ui;

use crate::ui::active_song_info::render_active_song_info;
use crate::ui::debug;
use crate::ui::modal;
use crate::ui::song::render_song_list;
use crate::ui::utils::StatefulList;

#[derive(PartialEq)]
pub enum Controller {
    Main,
    AddToPlaylist,
    PlayPlaylist,
    ModifyPlaylist,
    FuzzyFinder,
}

impl std::fmt::Display for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Controller::Main => write!(f, "Normal"),
            Controller::AddToPlaylist => write!(f, "AddToPlaylist"),
            Controller::PlayPlaylist => write!(f, "PlayPlaylist"),
            Controller::ModifyPlaylist => write!(f, "Modify Playlist"),
            _ => write!(f, ""),
        }
    }
}

pub struct App {
    pub finder_data: crate::ui::fuzzy_finder::Data,
    pub items: StatefulList<song::SongInfo>,

    pub progress: u32,
    pub volume: i32,
    pub song_info: Option<song::SongInfo>,
    pub song_data: serde_json::Value,

    pub playlist_info: playlist::PlaylistInfo,
    pub debugger: debug::Debug,

    pub controller: Controller,
    tx: Sender<song::ActionData>,

    pause: bool,
    quit: bool,
}

impl App {
    pub fn new(_song_data: serde_json::Value, _tx: Sender<song::ActionData>) -> App {
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
            finder_data: crate::ui::fuzzy_finder::Data::new(),
            items: StatefulList::with_items(_items),

            progress: 0,
            volume: 50,
            song_info: None,
            song_data: _song_data,

            playlist_info: playlist::PlaylistInfo::new("None"),
            debugger: debug::Debug::new(),

            controller: Controller::Main,
            tx: _tx,

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
            return ();
        }

        if self.items.state.selected() == None {
            return ();
        }

        let new_song_info = self.items.items[self.items.state.selected().unwrap()].clone();

        self.progress = 0;
        self.song_info = Some(new_song_info.clone());
        if self.pause {
            self.toggle_pause_song();
        }
        self.tx.send(song::ActionData {
            action: song::Action::AddSong,
            data: song::DataType::SongInfo(new_song_info),
        });
    }

    fn add_to_playlist(&mut self) {
        if self.items.state.selected() != None {
            self.finder_data
                .reset(playlist::playlist_names(), playlist::add_song_to_playlist);
            self.controller = Controller::AddToPlaylist;
        }
    }

    fn play_playlist(&mut self) {
        self.finder_data
            .reset(playlist::playlist_names(), playlist::play_playlist);
        self.controller = Controller::PlayPlaylist;
    }

    fn modify_playlist(&mut self) {
        self.finder_data
            .reset(playlist::playlist_names(), playlist::modify_playlist);
        self.controller = Controller::FuzzyFinder;
    }

    fn playback_playlist(&mut self) {
        if self.playlist_info.index >= self.playlist_info.songs.len() {
            self.playlist_info = playlist::PlaylistInfo::new("None");
            return ();
        }

        let new_song_info =
            SongInfo::new(self.playlist_info.songs[self.playlist_info.index].clone());

        self.progress = 0;
        self.song_info = Some(new_song_info.clone());
        self.tx.send(song::ActionData {
            action: song::Action::AddSong,
            data: song::DataType::SongInfo(new_song_info),
        });
        self.playlist_info.index += 1;
    }

    fn change_volume(&mut self, amount: i32) {
        self.volume = (self.volume + amount).max(0).min(100);
        self.tx.send(song::ActionData {
            action: song::Action::Volume,
            data: song::DataType::Int(self.volume),
        });
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

    pub fn current_controller(&self) -> String {
        self.controller.to_string().clone()
    }
}

pub fn setup(tx: Sender<song::ActionData>) -> Result<(), Box<dyn Error>> {
    let song_data = data::song_data()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let app = App::new(song_data, tx);
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
            Controller::AddToPlaylist => {
                ui::playlist::controller_add_to_playlist::<B>(&mut app, tick_rate, &mut last_tick)
            }
            Controller::PlayPlaylist => {
                ui::playlist::controller_play_playlist::<B>(&mut app, tick_rate, &mut last_tick)
            }
            Controller::ModifyPlaylist => {
                ui::playlist::controller_modify_playlist::<B>(&mut app, tick_rate, &mut last_tick)
            }
            Controller::FuzzyFinder => {
                ui::fuzzy_finder::controller::<B>(&mut app, tick_rate, &mut last_tick)
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
                KeyCode::Char('P') => app.play_playlist(),
                // Directly on Song
                KeyCode::Char('w') => app.change_volume(5),
                KeyCode::Char('b') => app.change_volume(-5),
                KeyCode::Char(' ') => app.toggle_pause_song(),
                // Change Mode
                KeyCode::Char('m') => app.modify_playlist(),
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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

    if app.controller == Controller::AddToPlaylist {
        ui::playlist::render_popup_add_to_playlist(f, app);
    } else if app.controller == Controller::PlayPlaylist {
        ui::playlist::render_popup_play_playlist(f, app);
    } else if app.controller == Controller::ModifyPlaylist {
        ui::playlist::render_modify_playlist(f, app, chunks[0], right_chunks[0]);
    } else if app.controller == Controller::FuzzyFinder {
        ui::fuzzy_finder::render_popup(f, app, "Fuzzy Find");
    }

    modal::render_modal(f, app, main_chunks[1]);
}
