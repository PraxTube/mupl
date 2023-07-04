/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
use crossterm::event::{self, Event, KeyCode};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use strsim::levenshtein;
use unicode_width::UnicodeWidthStr;

use crate::ui::utils;

pub struct Data {
    input: String,
    possible_matches: Vec<String>,
    stateful_matches: utils::StatefulList<String>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            input: String::new(),
            possible_matches: Vec::new(),
            stateful_matches: utils::StatefulList::with_items(Vec::new()),
        }
    }

    fn push_input(&mut self, c: char) {
        self.input.push(c);
        self.recalculate_matches();
    }

    fn pop_input(&mut self) {
        self.input.pop();
        self.recalculate_matches();
    }

    pub fn reset(&mut self, _possible_matches: Vec<String>) {
        self.possible_matches = _possible_matches.clone();
        self.stateful_matches = utils::StatefulList::with_items(_possible_matches);
    }

    fn recalculate_matches(&mut self) {
        self.possible_matches.sort_by(|a, b| {
            let a_distance = levenshtein(&self.input, a);
            let b_distance = levenshtein(&self.input, b);

            // Sort in descending order (more similar first)
            a_distance.cmp(&b_distance)
        });
        let mut sorted_and_filtered: Vec<String> = self
            .possible_matches
            .iter()
            .filter(|s| levenshtein(&self.input, s) <= 5)
            .cloned()
            .collect();

        sorted_and_filtered.sort_by(|a, b| {
            let a_distance = levenshtein(&self.input, a);
            let b_distance = levenshtein(&self.input, b);

            // Sort in descending order (more similar first)
            a_distance.cmp(&b_distance)
        });

        self.stateful_matches = utils::StatefulList::with_items(sorted_and_filtered);
    }
}

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut crate::terminal::App, title: &str) {
    let block = Block::default().title(title).borders(Borders::ALL);
    let area = utils::centered_rect(50, 30, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(area);

    let input = Paragraph::new(app.finder_data.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(input, chunks[0]);
    f.set_cursor(
        // Put cursor past the end of the input text
        chunks[0].x + app.finder_data.input.width() as u16,
        // Move one line down, from the border to the input line
        chunks[0].y,
    );
    let items: Vec<ListItem> = app
        .finder_data
        .stateful_matches
        .items
        .iter()
        .map(|i| {
            let song_body = Spans::from(Span::styled(i, Style::default()));
            ListItem::new(song_body).style(Style::default())
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(
        items,
        chunks[2],
        &mut app.finder_data.stateful_matches.state,
    );
}

pub fn controller<B: Backend>(
    app: &mut crate::terminal::App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    app.finder_data.push_input(c);
                }
                KeyCode::Backspace => {
                    app.finder_data.pop_input();
                }
                KeyCode::Tab => {
                    app.finder_data.stateful_matches.next();
                }
                KeyCode::BackTab => {
                    app.finder_data.stateful_matches.previous();
                }
                KeyCode::Enter => {}
                KeyCode::Esc => app.main_controller(),
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
