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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use unicode_width::UnicodeWidthStr;

use crate::ui::utils;

pub struct Data {
    input: String,
    pub matches: Vec<String>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            input: String::new(),
            matches: Vec::new(),
        }
    }
}

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut crate::terminal::App, title: &str) {
    let block = Block::default().title(title).borders(Borders::ALL);
    let area = utils::centered_rect(50, 30, f.size());
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
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
                    app.finder_data.input.push(c);
                }
                KeyCode::Backspace => {
                    app.finder_data.input.pop();
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
