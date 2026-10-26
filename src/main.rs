use std::{env::current_dir, io::Result};

use crossterm::terminal;

use crate::{deck::SpaceDeck, tui::App};

mod deck;
mod tui;

fn main() -> Result<()>{
    let dir = current_dir()?;
    let mut deck = SpaceDeck::new(dir);

    // match deck.size_cal() {
    //     Ok(size) => deck.print(1),
    //     Err(e) => eprintln!("Error: {e}")
    // }

    ratatui::run(|terminal| App::default().run(terminal))?;

    Ok(())
}
