#![allow(dead_code)]

use std::{
    io::Result,
    sync::{Arc, RwLock},
};

use crate::{progress::Progress, tui::App};
use clap::{Parser, Subcommand};

mod deck;
mod progress;
mod tui;

#[derive(Parser)]
#[command(name = "sd")]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand)]
enum CliCommand {
    Headless,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(CliCommand::Headless) => {
            let progress = Arc::new(RwLock::new(Progress::default()));
            let app = App::new(progress);
            _ = app.space_deck.write().unwrap().size_cal();
        }
        None => {
            let progress = Arc::new(RwLock::new(Progress::default()));
            let mut app = App::new(progress);
            app.read_disk();
            ratatui::run(|terminal| app.run(terminal))?;
        }
    }

    Ok(())
}
