use std::{
    env,
    io::Result,
    sync::{Arc, RwLock},
    thread,
};

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

use crate::{
    deck::{Entry, SpaceDeck},
    progress::Progress,
};

#[derive(Default)]
pub struct App {
    exit: bool,
    pub space_deck: Arc<RwLock<SpaceDeck>>,
    cursur: usize,
    progress: Arc<RwLock<Progress>>,
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
    pub fn new(progress: Arc<RwLock<Progress>>) -> Self {
        let dir = env::current_dir();

        match dir {
            Ok(dir) => {
                let progress_deck = Arc::clone(&progress);
                let sd = SpaceDeck::new(dir, progress_deck);

                Self {
                    exit: false,
                    space_deck: Arc::new(RwLock::new(sd)),
                    cursur: 0,
                    progress,
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

    pub fn read_disk(&mut self) {
        let sd = Arc::clone(&self.space_deck);
        let _ = thread::spawn(move || {
            let mut guard = sd.write().unwrap();
            if let Err(error) = guard.size_cal() {
                eprintln!("Error: {error}");
            };
            drop(guard);
        });
    }

    fn move_cursor(&mut self, down: bool) {
        let sd = Arc::clone(&self.space_deck);
        let deck = sd.read().unwrap();

        if down {
            if self.cursur == deck.current_folder().unwrap().entries.len() - 1 {
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
        let sd = Arc::clone(&self.space_deck);
        let mut deck = sd.write().unwrap();
        // ensure we dont enter a non existant dir
        if deck.current_folder().unwrap().entries.is_empty() {
            return;
        }

        // if file
        if !matches!(
            deck.current_folder().unwrap().entries.get(self.cursur),
            Some(Entry::Folder(_))
        ) {
            return;
        }

        deck.pointer.push(self.cursur);
        self.cursur = 0;
        deck.current_folder_mut().unwrap().sort();
        drop(deck);
    }

    fn exit_dir(&mut self) {
        let sd = Arc::clone(&self.space_deck);
        let mut deck = sd.write().unwrap();

        deck.pointer.pop();
        self.cursur = 0;
        drop(deck);
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let outline = Block::bordered()
            .title("space-deck")
            .title_alignment(Alignment::Center);

        let inner = outline.inner(area);

        frame.render_widget(outline, area);

        let sd = Arc::clone(&self.space_deck);

        match sd.try_read() {
            Ok(deck) => self.draw_allocation(frame, area, &deck),
            _ => {
                self.draw_read_progression(frame, inner);
            }
        }
    }

    fn draw_allocation(&self, frame: &mut Frame, area: Rect, deck: &SpaceDeck) {
        let [header, body] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

        let header_text = Text::from(deck.current_folder().unwrap().dir.display().to_string());

        frame.render_widget(
            Paragraph::new(header_text).wrap(Wrap { trim: false }),
            header,
        );

        let total = deck.current_folder().unwrap().size.max(1);

        let constraints: Vec<Constraint> = deck
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
            .zip(deck.current_folder().unwrap().entries.iter())
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

    fn draw_read_progression(&self, frame: &mut Frame, area: Rect) {
        let [header, body, errors] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .areas(area);

        let header_text = Text::from("Reading disk...");
        frame.render_widget(header_text, header);

        let scanned_size = ByteSize::b(self.progress.read().unwrap().progress)
            .display()
            .to_string();
        let body_text = Text::from(format!("Scanned: {scanned_size}"));

        frame.render_widget(body_text, body);

        let progress = self.progress.read().unwrap();

        let error_constraints: Vec<Constraint> = progress
            .errors
            .iter()
            .skip(progress.errors.len() - errors.height as usize)
            .map(|_| Constraint::Length(1))
            .collect();
        drop(progress);

        let error_rows = Layout::vertical(error_constraints).split(errors);

        let progress = self.progress.read().unwrap();
        for (error, row) in progress
            .errors
            .iter()
            .skip(progress.errors.len() - errors.height as usize)
            .zip(error_rows.iter())
        {
            let text = Text::from(error.to_string());
            frame.render_widget(text, *row);
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if !event::poll(std::time::Duration::from_millis(100))? {
            return Ok(());
        }

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                let sd = Arc::clone(&self.space_deck);
                let deck = sd.read().unwrap();

                if !deck.reading {
                    drop(deck);
                    match key_event.code {
                        KeyCode::Esc => self.exit = true,
                        KeyCode::Down => self.move_cursor(true),
                        KeyCode::Up => self.move_cursor(false),
                        KeyCode::Enter => self.enter_dir(),
                        KeyCode::Backspace => self.exit_dir(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
