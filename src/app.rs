use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use thiserror::Error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::image::Image;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error in terminal initialization.")]
    InitTerm(#[source] std::io::Error),
    #[error("IO error in terminal finalization.")]
    FinTerm(#[source] std::io::Error),
    #[error("IO error in drawing process.")]
    Draw(#[source] std::io::Error),
    #[error("IO error in polling key event.")]
    KeyEvent(#[source] std::io::Error),
}

#[derive(Clone, PartialEq, Debug)]
pub struct App {
    image_txt: Text<'static>,
}

impl App {
    pub fn new(img: &Image) -> Self {
        Self {
            image_txt: img.into(),
        }
    }

    fn ui(&self, f: &mut Frame<impl Backend>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let canvas = Block::default().title("Canvas").borders(Borders::ALL);

        // dbg!(&text);
        let img = Paragraph::new(self.image_txt.clone())
            .block(canvas)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        f.render_widget(img, chunks[0]);
    }

    pub fn run(&self) -> Result<(), AppError> {
        // setup terminal
        enable_raw_mode().map_err(AppError::InitTerm)?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(AppError::InitTerm)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(AppError::InitTerm)?;

        // create app and run it
        let res = self.main_loop(&mut terminal);

        // restore terminal
        disable_raw_mode().map_err(AppError::FinTerm)?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen,).map_err(AppError::FinTerm)?;
        terminal.show_cursor().map_err(AppError::FinTerm)?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    fn main_loop(&self, terminal: &mut Terminal<impl Backend>) -> Result<(), AppError> {
        loop {
            terminal.draw(|f| self.ui(f)).map_err(AppError::Draw)?;
            if let Event::Key(key) = event::read().map_err(AppError::KeyEvent)? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}
