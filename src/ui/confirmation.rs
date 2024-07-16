use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::{
    io,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::ui::utils;

pub struct Data {
    message: String,
    title: String,

    highlight_pos: bool,

    pub pos_result_func: Option<fn(&mut crate::ui::terminal::App) -> ()>,
    pub neg_result_func: Option<fn(&mut crate::ui::terminal::App) -> ()>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            message: String::new(),
            title: String::new(),

            highlight_pos: true,

            pos_result_func: None,
            neg_result_func: None,
        }
    }

    pub fn reset(
        &mut self,
        message: String,
        title: String,
        pos_result_func: fn(&mut crate::ui::terminal::App) -> (),
        neg_result_func: fn(&mut crate::ui::terminal::App) -> (),
    ) {
        self.message = message;
        self.title = title;
        self.pos_result_func = Some(pos_result_func);
        self.neg_result_func = Some(neg_result_func);
    }
}

fn calculate_sub_chunks(app: &mut crate::terminal::App, area: Rect) -> Rc<[Rect]> {
    let constraints = match app.confirmation_data.highlight_pos {
        true => vec![
            Constraint::Length(20),
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(5),
            Constraint::Length(19),
        ],
        false => vec![
            Constraint::Length(19),
            Constraint::Length(4),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(20),
        ],
    };
    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);
    sub_chunks
}

fn neg_paragraph(app: &mut crate::ui::terminal::App) -> Paragraph {
    match app.confirmation_data.highlight_pos {
        true => Paragraph::new("NO")
            .style(Style::default())
            .block(Block::default().borders(Borders::NONE)),
        false => Paragraph::new("<NO>")
            .style(Style::default().fg(Color::Red).bg(Color::Black))
            .block(Block::default().borders(Borders::NONE)),
    }
}

fn pos_paragraph(app: &mut crate::ui::terminal::App) -> Paragraph {
    match app.confirmation_data.highlight_pos {
        true => Paragraph::new("<YES>")
            .style(Style::default().fg(Color::Red).bg(Color::Black))
            .block(Block::default().borders(Borders::NONE)),
        false => Paragraph::new("YES")
            .style(Style::default())
            .block(Block::default().borders(Borders::NONE)),
    }
}

pub fn render_popup(f: &mut Frame, app: &mut crate::ui::terminal::App) {
    let title = app.confirmation_data.title.clone();
    let block = Block::default().title(title).borders(Borders::ALL);
    let area = utils::centered_rect(40, 20, f.size());
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

    let input = Paragraph::new(format!("{}", app.confirmation_data.message))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(input, chunks[0]);

    let sub_chunks = calculate_sub_chunks(app, chunks[2]);
    f.render_widget(neg_paragraph(app), sub_chunks[1]);
    f.render_widget(pos_paragraph(app), sub_chunks[3]);
}

pub fn controller(
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
                KeyCode::Char('h') => app.confirmation_data.highlight_pos = false,
                KeyCode::Char('l') => app.confirmation_data.highlight_pos = true,
                KeyCode::Tab => {
                    app.confirmation_data.highlight_pos = !app.confirmation_data.highlight_pos
                }
                KeyCode::Char('q') => neg_response(app),
                KeyCode::Enter => response(app),
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

fn response(app: &mut crate::terminal::App) {
    match app.confirmation_data.highlight_pos {
        true => pos_response(app),
        false => neg_response(app),
    }
}

fn pos_response(app: &mut crate::terminal::App) {
    match app.confirmation_data.pos_result_func {
        Some(func) => func(app),
        None => app.debugger.print("No pos result func set."),
    }
}

fn neg_response(app: &mut crate::terminal::App) {
    match app.confirmation_data.neg_result_func {
        Some(func) => func(app),
        None => app.debugger.print("No neg result func set."),
    }
}
