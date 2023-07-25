use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::{
    io,
    time::{Duration, Instant},
};

use unicode_width::UnicodeWidthStr;

use crate::ui::utils;

pub struct Data {
    input: String,
    pub output: String,
    title: String,
    message: String,

    pub result_func: Option<fn(&mut crate::ui::terminal::App) -> ()>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            input: String::new(),
            output: String::new(),
            title: String::new(),
            message: String::new(),

            result_func: None,
        }
    }

    fn push_input(&mut self, c: char) {
        self.input.push(c);
    }

    fn pop_input(&mut self) {
        self.input.pop();
    }

    pub fn reset(
        &mut self,
        title: &str,
        message: &str,
        result_func: fn(&mut crate::ui::terminal::App) -> (),
    ) {
        self.input = String::new();
        self.title = title.to_string();
        self.message = message.to_string();
        self.result_func = Some(result_func);
    }

    fn enter(&mut self) {
        self.output = self.input.clone();
    }
}

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut crate::terminal::App) {
    let title = app.text_prompt_data.title.clone();
    let block = Block::default().title(title).borders(Borders::ALL);
    let area = utils::centered_rect(50, 30, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(area);

    let message = Paragraph::new(format!("{}", app.text_prompt_data.message))
        .style(Style::default())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(message, chunks[0]);

    let input = Paragraph::new(format!("> {}", app.text_prompt_data.input))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(input, chunks[2]);
    f.set_cursor(
        chunks[2].x + app.text_prompt_data.input.width() as u16 + 2,
        chunks[2].y,
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
                    app.text_prompt_data.push_input(c);
                }
                KeyCode::Backspace => {
                    app.text_prompt_data.pop_input();
                }
                KeyCode::Enter => {
                    app.text_prompt_data.enter();
                    match app.text_prompt_data.result_func {
                        Some(func) => func(app),
                        None => {}
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
