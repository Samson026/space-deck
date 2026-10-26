use std::{env::current_dir, io::Result};

use crate::deck::SpaceDeck;

mod deck;

fn main() -> Result<()>{
    let dir = current_dir()?;
    let mut deck = SpaceDeck::new(dir);

    match deck.size_cal() {
        Ok(size) => deck.print(2),
        Err(e) => eprintln!("Error: {e}")
    }

    Ok(())
}
