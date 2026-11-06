#![allow(dead_code)]

use std::{
    io::Result,
    sync::{Arc, RwLock},
};

use crate::{progress::Progress, tui::App};

mod deck;
mod progress;
mod tui;

fn main() -> Result<()> {
    let progress = Arc::new(RwLock::new(Progress::default()));
    let app_progress = Arc::clone(&progress);
    let mut app = App::new(app_progress);

    app.read_disk();
    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
