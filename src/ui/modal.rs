use tui::{backend::Backend, layout::Rect, widgets::Paragraph, Frame};

use crate::ui::terminal::App;

pub fn render_modal<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let mode = app.current_controller();
    let paragraph_info = Paragraph::new(mode);
    f.render_widget(paragraph_info, chunk);
}
