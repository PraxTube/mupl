use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::terminal::App;
use std::collections::VecDeque;

pub struct Debug {
    pub messages: VecDeque<String>,
}

impl Debug {
    pub fn new() -> Debug {
        Debug {
            messages: VecDeque::new(),
        }
    }

    pub fn print(&mut self, message: &str) {
        self.messages.push_back(message.to_string());
        if self.messages.len() > 10 {
            self.messages.pop_front();
        }
    }
}

pub fn render_active_song_info(f: &mut Frame, app: &mut App, chunk: Rect) {
    let block = Block::default().title("Debug").borders(Borders::ALL);
    f.render_widget(block, chunk);

    let chunk = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)])
        .split(chunk);
    let mut debug_string = String::new();

    for msg in &app.debugger.messages {
        debug_string += msg;
        debug_string += "\n";
    }
    let paragraph_info = Paragraph::new(debug_string);
    f.render_widget(paragraph_info, chunk[0]);
}
