use std::{env, io::Result};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
};

use crate::deck::SpaceDeck;

#[derive(Default)]
pub struct App {
    exit: bool,
    space_deck: SpaceDeck,
}

const COLOURS: [Color; 10] = [
    Color::Blue,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
    Color::LightBlue,
    Color::LightRed,
    Color::LightGreen,
    Color::LightMagenta,
];

fn get_colour(i: usize) -> Color {
    COLOURS[i % COLOURS.len()]
}

impl App {
    pub fn new() -> Self {
        let dir = env::current_dir();

        match dir {
            Ok(dir) => {
                let mut sd = SpaceDeck::new(dir);

                println!("Reading...");

                if let Err(e) = sd.size_cal() {
                    eprintln!("Error: {e}");
                }

                Self {
                    exit: false,
                    space_deck: sd,
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                App::default()
            }
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let outline = Block::bordered()
            .title("space-deck")
            .title_alignment(Alignment::Center);

        let inner = outline.inner(area);

        frame.render_widget(outline, area);

        self.draw_allocation(frame, inner);
    }

    fn draw_allocation(&self, frame: &mut Frame, area: Rect) {
        let total = self.space_deck.head.size.max(1);

        let constraints: Vec<Constraint> = self
            .space_deck
            .head
            .folders
            .iter()
            .map(|folder| {
                let weight = ((folder.size.max(1) * 1000 / total) as u16).max(1);

                Constraint::Fill(weight)
            })
            .collect();

        let cols = Layout::horizontal(constraints).split(area);

        for (i, (col, folder)) in cols
            .iter()
            .zip(self.space_deck.head.folders.iter())
            .enumerate()
        {
            let title = folder.dir.file_name().unwrap_or_default().to_string_lossy();
            let border = Block::bordered().title(title);
            let inner = border.inner(*col);

            frame.render_widget(border, *col);

            frame
                .buffer_mut()
                .set_style(inner, Style::new().bg(get_colour(i)));
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event)
                if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Esc =>
            {
                self.exit = true
            }
            _ => {}
        }
        Ok(())
    }
}
