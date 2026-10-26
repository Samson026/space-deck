use std::io::Result;

use crossterm::{event::{self, Event, KeyCode, KeyEventKind}, style::SetStyle};
use ratatui::{DefaultTerminal, Frame, layout::Alignment, widgets::Block};

use crate::deck::SpaceDeck;

#[derive(Default)]
pub struct App {
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(Block::bordered().title("space-deck").title_alignment(Alignment::Center), frame.area());
        


    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.exit = true,
                    _ => {}
                }
            }
            _ => {}
            
        }
        Ok(())
    }
}