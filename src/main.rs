#![allow(dead_code)]

use std::io::Result;

use crate::tui::App;

mod deck;
mod tui;

fn main() -> Result<()> {
    let mut app = App::new();

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
