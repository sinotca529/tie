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
    image: Image,
    cmd_stream: T,
    cursor_coord: (usize, usize),
}

impl<CS: CommandStream> App<CS> {
    pub fn new(img: Image, cmd_stream: CS) -> Self {
        App {
            image: img,
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
        let img = Paragraph::new(self.image.clone().into_text_with_cursor(self.cursor_coord))
            .block(canvas)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        f.render_widget(img, chunks[0]);
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
                    .min(self.image.height() as usize - 1);
            }
            crate::command::Direction::Left => {
                self.cursor_coord.0 = self.cursor_coord.0.saturating_sub(1);
            }
            crate::command::Direction::Right => {
                self.cursor_coord.0 = self
                    .cursor_coord
                    .0
                    .saturating_add(1)
                    .min(self.image.width() as usize - 1);
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
    use crate::command::{self, programmed::ProgrammedEvent};

    use super::*;
    #[test]
    fn test_app_run_without_error() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let cs = ProgrammedEvent::new(vec![
            Command::Unknown,
            Command::Unknown,
            Command::Unknown,
            Command::Quit,
            Command::Unknown,
            Command::Unknown,
        ]);
        let mut app = App::new(img, cs);
        assert!(matches!(app.run(), Ok(_)));
    }

    #[test]
    fn test_move_cursor() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let (w, h) = (img.width() as usize, img.height() as usize);
        let mut cmds = vec![Command::Direction(command::Direction::Right); w - 1];
        cmds.append(&mut vec![
            Command::Direction(command::Direction::Down);
            h - 1
        ]);

        cmds.push(Command::Quit);
        let cs = ProgrammedEvent::new(cmds);
        let mut app = App::new(img, cs);
        assert_eq!(app.cursor_coord, (0, 0));
        app.run().unwrap();
        assert_eq!(app.cursor_coord, (w - 1, h - 1));
    }

    #[test]
    fn boundary_test_move_cursor_left() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let mut cmds = vec![Command::Direction(command::Direction::Left)];
        cmds.push(Command::Quit);
        let cs = ProgrammedEvent::new(cmds);
        let mut app = App::new(img, cs);
        assert_eq!(app.cursor_coord, (0, 0));
        app.run().unwrap();
        assert_eq!(app.cursor_coord, (0, 0));
    }
    #[test]
    fn boundary_test_move_cursor_right() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let img_width = img.width() as usize;
        let mut cmds = vec![Command::Direction(command::Direction::Right); img_width + 5];
        cmds.push(Command::Quit);
        let cs = ProgrammedEvent::new(cmds);
        let mut app = App::new(img, cs);
        assert_eq!(app.cursor_coord, (0, 0));
        app.run().unwrap();
        assert_eq!(app.cursor_coord, (img_width - 1, 0));
    }
    #[test]
    fn boundary_test_move_cursor_up() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let mut cmds = vec![Command::Direction(command::Direction::Up)];
        cmds.push(Command::Quit);
        let cs = ProgrammedEvent::new(cmds);
        let mut app = App::new(img, cs);
        assert_eq!(app.cursor_coord, (0, 0));
        app.run().unwrap();
        assert_eq!(app.cursor_coord, (0, 0));
    }
    #[test]
    fn boundary_test_move_cursor_down() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let img_height = img.height() as usize;
        let mut cmds = vec![Command::Direction(command::Direction::Down); img_height + 5];
        cmds.push(Command::Quit);
        let cs = ProgrammedEvent::new(cmds);
        let mut app = App::new(img, cs);
        assert_eq!(app.cursor_coord, (0, 0));
        app.run().unwrap();
        assert_eq!(app.cursor_coord, (0, img_height - 1));
    }
}
