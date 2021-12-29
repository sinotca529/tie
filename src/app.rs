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
    image: Text<'static>,
    cmd_stream: T,
    cursor_coord: (usize, usize),
}

impl<CS: CommandStream> App<CS> {
    pub fn new(img: &Image, cmd_stream: CS) -> Self {
        App {
            image: img.into(),
            cmd_stream,
            cursor_coord: (0, 0),
        }
    }

    fn ui(&self, f: &mut Frame<impl Backend>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let canvas = Block::default().title("Canvas").borders(Borders::ALL);

        // dbg!(&text);
        let img = Paragraph::new(self.image_with_cursor())
            .block(canvas)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        f.render_widget(img, chunks[0]);
    }

    fn image_with_cursor(&self) -> Text<'static> {
        assert!(self.cursor_coord.0 < self.image.width() / 2);
        assert!(self.cursor_coord.1 < self.image.height());

        let mut cloned_img = self.image.clone();

        let span = &mut cloned_img.lines[self.cursor_coord.1].0[self.cursor_coord.0];

        if let Some(Color::Rgb(r, g, b)) = span.style.bg {
            let opposite_color = Color::Rgb(255 - r, 255 - g, 255 - b);

            span.content = "[]".into();
            span.style = span.style.fg(opposite_color);
            cloned_img
        } else {
            unreachable!()
        }
    }

    fn move_cursor(&mut self, dir: crate::command::Direction) {
        match dir {
            crate::command::Direction::Up => {
                self.cursor_coord.1 = self.cursor_coord.1.saturating_sub(1);
            }
            crate::command::Direction::Down => {
                self.cursor_coord.1 = self
                    .cursor_coord
                    .1
                    .saturating_add(1)
                    .min(self.image.height() - 1);
            }
            crate::command::Direction::Left => {
                self.cursor_coord.0 = self.cursor_coord.0.saturating_sub(1);
            }
            crate::command::Direction::Right => {
                self.cursor_coord.0 = self
                    .cursor_coord
                    .0
                    .saturating_add(1)
                    .min(self.image.width() / 2 - 1);
            }
        }
    }
}

impl<CS> App<CS>
where
    CS: CommandStream,
    CS::Error: std::error::Error + std::fmt::Debug,
{
    pub fn run(&mut self) -> Result<(), AppError<CS::Error>> {
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

    fn main_loop(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
    ) -> Result<(), AppError<CS::Error>> {
        loop {
            terminal.draw(|f| self.ui(f)).map_err(AppError::Draw)?;
            match self.cmd_stream.read().map_err(AppError::ReadCommand)? {
                Command::Quit => break,
                Command::Unknown => {}
                Command::Direction(dir) => self.move_cursor(dir),
            }
        }
        Ok(())
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
        let mut app = App::new(&img, cs);
        assert!(matches!(app.run(), Ok(_)));
    }
}
