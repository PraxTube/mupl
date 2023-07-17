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
    pub output: String,
    title: String,

    possible_matches: Vec<String>,
    stateful_matches: utils::StatefulList<String>,

    pub result_func: Option<fn(&mut crate::ui::terminal::App) -> ()>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            input: String::new(),
            output: String::new(),
            title: String::new(),

            possible_matches: Vec::new(),
            stateful_matches: utils::StatefulList::with_items(Vec::new()),

            result_func: None,
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

    pub fn reset(
        &mut self,
        title: String,
        possible_matches: Vec<String>,
        result_func: fn(&mut crate::ui::terminal::App) -> (),
    ) {
        self.input = String::new();
        self.title = title;
        self.possible_matches = possible_matches.clone();
        self.result_func = Some(result_func);
        self.stateful_matches = utils::StatefulList::with_items(possible_matches);
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
        self.stateful_matches.next();
    }

    fn check_finding(&mut self) -> bool {
        if self.stateful_matches.items.len() == 0 {
            return false;
        }
        if self.stateful_matches.state.selected() == None {
            return false;
        }

        self.output =
            self.stateful_matches.items[self.stateful_matches.state.selected().unwrap()].clone();
        true
    }
}

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut crate::terminal::App) {
    let title = app.finder_data.title.clone();
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

    let input = Paragraph::new(format!("> {}", app.finder_data.input))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(input, chunks[0]);
    f.set_cursor(
        // Put cursor past the end of the input text
        chunks[0].x + app.finder_data.input.width() as u16 + 2,
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
                KeyCode::Enter => {
                    if app.finder_data.check_finding() {
                        match app.finder_data.result_func {
                            Some(func) => func(app),
                            None => {}
                        }
                    }
                }
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
