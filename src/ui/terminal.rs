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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, LineGauge, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::data;
use crate::load;
use crate::song::SongInfo;
use crate::ui::playlist;
use crate::ui::utils::StatefulList;

enum Controller {
    Main,
    Playlist,
}

pub struct App {
    pub finder_data: crate::ui::fuzzy_finder::Data,
    items: StatefulList<SongInfo>,

    progress: u32,
    song_info: Option<SongInfo>,
    songs: Vec<std::path::PathBuf>,
    song_data: serde_json::Value,

    playlist_popup: bool,

    controller: Controller,
    tx: Sender<SongInfo>,

    quit: bool,
}

impl App {
    pub fn new(
        _songs: Vec<std::path::PathBuf>,
        _song_data: serde_json::Value,
        _tx: Sender<SongInfo>,
    ) -> App {
        let mut _items: Vec<SongInfo> = Vec::new();
        for song in &_songs {
            _items.push(SongInfo::new(song));
        }
        App {
            finder_data: crate::ui::fuzzy_finder::Data::new(),
            items: StatefulList::with_items(_items),

            progress: 0,
            song_info: None,
            songs: _songs,
            song_data: _song_data,

            playlist_popup: false,

            controller: Controller::Main,
            tx: _tx,

            quit: false,
        }
    }

    pub fn get_progress(&self) -> u32 {
        self.progress
    }

    pub fn on_tick(&mut self) {
        match &self.song_info {
            Some(info) => {
                self.progress += 1;
                if self.progress > info.duration {
                    self.progress = 0;
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
        self.tx.send(new_song_info);
    }

    fn add_to_playlist(&mut self) {
        if self.items.state.selected() != None {
            self.playlist_popup = true;
            self.reset_finder(playlist::playlist_names());
            self.controller = Controller::Playlist;
        }
    }

    fn reset_finder(&mut self, matches: Vec<String>) {
        self.finder_data.matches = matches;
    }

    pub fn main_controller(&mut self) {
        self.playlist_popup = false;
        self.controller = Controller::Main;
    }
}

pub fn setup(tx: Sender<SongInfo>) -> Result<(), Box<dyn Error>> {
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
            Controller::Playlist => playlist::controller::<B>(&mut app, tick_rate, &mut last_tick),
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

fn render_song_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let song_body = Spans::from(Span::styled(&i.name, Style::default()));
            ListItem::new(song_body).style(Style::default())
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Song List"))
        .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunk, &mut app.items.state);
}

fn render_play_song<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect, song_info: SongInfo) {
    let playing_song_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(chunk);

    let paragraph_info = Paragraph::new(format!(
        "\nName: {}\nFile: {}",
        app.song_data[&song_info.name]["name"], app.song_data[&song_info.name]["artist"][0]
    ))
    .alignment(Alignment::Left);
    f.render_widget(paragraph_info, playing_song_chunks[0]);

    let label = format!(
        "{}/{} - ({:.0}%)",
        format_time(app.progress),
        format_time(song_info.duration),
        app.progress as f64 / song_info.duration as f64 * 100.0
    );
    let ratio: f64 = (app.get_progress() as f64 / song_info.duration as f64)
        .max(0.0)
        .min(1.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(ratio)
        .label(label.clone());
    f.render_widget(gauge, playing_song_chunks[1]);

    let gauge = LineGauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(label);
    f.render_widget(gauge, playing_song_chunks[2]);
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
        Some(info) => render_play_song(f, app, chunks[1], info.clone()),
        None => {}
    }

    if app.playlist_popup {
        playlist::render_popup(f, app);
    }
}

fn format_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
