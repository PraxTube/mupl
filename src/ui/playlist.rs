use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::Backend, Frame};

use crate::ui::fuzzy_finder;
use crate::ui::terminal::App;

pub fn render_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    fuzzy_finder::render_popup(f, app, "Song PlayList");
}

pub fn controller<B: Backend>(
    app: &mut App,
    tick_rate: Duration,
    last_tick: &mut Instant,
) -> io::Result<()> {
    fuzzy_finder::controller::<B>(app, tick_rate, last_tick)
}
