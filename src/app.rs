use crossterm::{
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

use crate::{
    command::{Command, CommandStream},
    image::Image,
};

#[derive(Error, Debug)]
pub enum AppError<E: 'static + std::error::Error + std::fmt::Debug> {
    #[error("IO error in terminal initialization.")]
    InitTerm(#[source] std::io::Error),
    #[error("IO error in terminal finalization.")]
    FinTerm(#[source] std::io::Error),
    #[error("IO error in drawing process.")]
    Draw(#[source] std::io::Error),
    #[error("Error in read command.")]
    ReadCommand(#[source] E),
}

pub struct App<T: CommandStream> {
    image_txt: Text<'static>,
    cmd_stream: T,
}

impl<CS: CommandStream> App<CS> {
    pub fn new(img: &Image, cmd_stream: CS) -> Self {
        App {
            image_txt: img.into(),
            cmd_stream,
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
}

impl<CS> App<CS>
where
    CS: CommandStream,
    CS::Error: std::error::Error + std::fmt::Debug,
{
    pub fn run(&self) -> Result<(), AppError<CS::Error>> {
        // setup terminal
        enable_raw_mode().map_err(AppError::InitTerm)?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(AppError::InitTerm)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(AppError::InitTerm)?;

        // create app and run it
        self.main_loop(&mut terminal)?;

        // restore terminal
        disable_raw_mode().map_err(AppError::FinTerm)?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen,).map_err(AppError::FinTerm)?;
        terminal.show_cursor().map_err(AppError::FinTerm)?;

        Ok(())
    }

    fn main_loop(&self, terminal: &mut Terminal<impl Backend>) -> Result<(), AppError<CS::Error>> {
        loop {
            terminal.draw(|f| self.ui(f)).map_err(AppError::Draw)?;
            if let Command::Quit = self.cmd_stream.read().map_err(AppError::ReadCommand)? {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command::programmed::ProgrammedEvent;

    use super::*;
    #[test]
    fn app_run_without_error_test() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let cs = ProgrammedEvent::new(vec![
            Command::Unknown,
            Command::Unknown,
            Command::Unknown,
            Command::Quit,
            Command::Unknown,
            Command::Unknown,
        ]);
        let app = App::new(&img, cs);
        assert!(matches!(app.run(), Ok(_)));
    }
}
