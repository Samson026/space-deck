#![allow(dead_code)]

use std::io::Result;

use crate::tui::App;

mod deck;
mod tui;

fn main() -> Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))?;

    Ok(())
}
