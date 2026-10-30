use std::{env, io::Result};

use bytesize::ByteSize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{
        Color::{self, Green},
        Style,
    },
    text::Text,
    widgets::{Block, Paragraph, Wrap},
};

use crate::deck::{Entry, SpaceDeck};

#[derive(Default)]
pub struct App {
    exit: bool,
    space_deck: SpaceDeck,
    cursur: usize,
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
                    cursur: 0,
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

    fn move_cursor(&mut self, down: bool) {
        if down {
            if self.cursur == self.space_deck.current_folder().unwrap().entries.len() - 1 {
                self.cursur = 0
            } else {
                self.cursur += 1
            }
        } else {
            if self.cursur == 0 {
                return;
            }
            self.cursur -= 1
        }
    }

    fn enter_dir(&mut self) {
        // ensure we dont enter a non existant dir
        if self.space_deck.current_folder().unwrap().entries.is_empty() {
            return;
        }

        // if file
        if !matches!(
            self.space_deck
                .current_folder()
                .unwrap()
                .entries
                .get(self.cursur),
            Some(Entry::Folder(_))
        ) {
            return;
        }

        self.space_deck.pointer.push(self.cursur);
        self.cursur = 0;
        self.space_deck.current_folder_mut().unwrap().sort();
    }

    fn exit_dir(&mut self) {
        self.space_deck.pointer.pop();
        self.cursur = 0;
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
        let [header, body] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let header_text = Text::from(
            self.space_deck
                .current_folder()
                .unwrap()
                .dir
                .display()
                .to_string(),
        );

        frame.render_widget(
            Paragraph::new(header_text).wrap(Wrap { trim: false }),
            header,
        );

        let total = self.space_deck.current_folder().unwrap().size.max(1);

        let constraints: Vec<Constraint> = self
            .space_deck
            .current_folder()
            .unwrap()
            .entries
            .iter()
            .take(20)
            .map(|entry| {
                let weight = ((entry.size().max(1) * 1000 / total) as u16).max(1);

                let constraint: Constraint = if weight < 100 {
                    Constraint::Min(1)
                } else {
                    Constraint::Fill(weight)
                };

                constraint
            })
            .collect();

        let cols = Layout::vertical(constraints).split(body);

        for (i, (col, entry)) in cols
            .iter()
            .zip(self.space_deck.current_folder().unwrap().entries.iter())
            .enumerate()
        {
            let mut border: Block;
            match entry {
                Entry::Folder(folder) => {
                    let title = folder.dir.file_name().unwrap_or_default().to_string_lossy();
                    border = Block::bordered().title(title);
                }
                Entry::File(file) => {
                    let title =
                        file.dir.file_name().unwrap_or_default().to_string_lossy() + " - File";
                    border = Block::bordered().title(title);
                }
            }

            if i == self.cursur {
                border = border.border_style(Style::new().fg(Green));
            }

            let inner = border.inner(*col);

            frame.render_widget(border, *col);

            frame
                .buffer_mut()
                .set_style(inner, Style::new().bg(get_colour(i)));

            let size = ByteSize::b(entry.size()).display().to_string();
            let size_text = Text::from(size).centered();

            frame.render_widget(size_text, inner);
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.exit = true,
                    KeyCode::Down => self.move_cursor(true),
                    KeyCode::Up => self.move_cursor(false),
                    KeyCode::Enter => self.enter_dir(),
                    KeyCode::Backspace => self.exit_dir(),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}
