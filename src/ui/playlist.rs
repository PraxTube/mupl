use tui::{
    backend::Backend,
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::ui::terminal::App;
use crate::ui::utils;

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let block = Block::default()
        .title("Select Playlist")
        .borders(Borders::ALL);
    let area = utils::centered_rect(50, 30, f.size());
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);
}
