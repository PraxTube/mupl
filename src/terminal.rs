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
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, LineGauge, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::song::SongInfo;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct App {
    items: StatefulList<(SongInfo, usize)>,

    song_info: SongInfo,
    progress: u32,

    tx: Sender<SongInfo>,
}

impl App {
    pub fn new(_song_info: SongInfo, _tx: Sender<SongInfo>) -> App {
        App {
            items: StatefulList::with_items(vec![
                (SongInfo::new("tmp/wannabe.wav"), 0),
                (SongInfo::new("GangGangKawaii.wav"), 1),
                (SongInfo::new("dummy.wav"), 2),
            ]),

            song_info: _song_info,
            progress: 0,

            tx: _tx,
        }
    }

    pub fn get_progress(&self) -> u32 {
        self.progress
    }

    pub fn on_tick(&mut self) {
        self.progress += 1;
        if self.progress > self.song_info.duration {
            self.progress = 0;
        }
    }

    fn change_playing_song(&mut self) {
        if self.items.state.selected() == None {
            return ();
        }

        let new_song_info = self.items.items[self.items.state.selected().unwrap()]
            .0
            .clone();

        self.progress = 0;
        self.song_info = new_song_info.clone();
        self.tx.send(new_song_info);
    }
}

pub fn setup(song_info: SongInfo, tx: Sender<SongInfo>) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_secs(1);
    let app = App::new(song_info, tx);
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
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.items.unselect(),
                    KeyCode::Char('l') => app.items.unselect(),
                    KeyCode::Char('j') => app.items.next(),
                    KeyCode::Char('k') => app.items.previous(),
                    KeyCode::Enter => app.change_playing_song(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let song_body = Spans::from(Span::styled(&i.0.file, Style::default()));
            ListItem::new(song_body).style(Style::default())
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Song List"))
        .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    let block = Block::default().title("Playing Song").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

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
        .split(chunks[1]);

    let paragraph_info = Paragraph::new(format!(
        "\nName: {}\nFile: {}",
        app.song_info.name, app.song_info.file
    ))
    .alignment(Alignment::Left);
    f.render_widget(paragraph_info, playing_song_chunks[0]);

    let label = format!(
        "{}/{} - ({:.0}%)",
        format_time(app.progress),
        format_time(app.song_info.duration),
        app.progress as f64 / app.song_info.duration as f64 * 100.0
    );
    let ratio: f64 = (app.get_progress() as f64 / app.song_info.duration as f64)
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

fn format_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
