use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use thiserror::Error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use crate::{
    command::{Command, CommandStream},
    image::Image,
    widget::{canvas::Canvas, palette::Palette, Widget},
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
    cmd_stream: T,
    canvas: Canvas,
    palette: Palette,
}

impl<CS: CommandStream> App<CS> {
    pub fn new(img: Image, cmd_stream: CS) -> Self {
        App {
            cmd_stream,
            canvas: Canvas::new(img),
            palette: Palette::default(),
        }
    }

    fn render(&self, f: &mut Frame<impl Backend>) {
        let chunks1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(f.size().height - 3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        let chunks2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(chunks1[0]);

        self.palette.render(f, chunks2[0]);
        self.canvas.render(f, chunks2[1]);
        self.cmd_stream.render(f, chunks1[1]);
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
            terminal.draw(|f| self.render(f)).map_err(AppError::Draw)?;

            match self.cmd_stream.read().map_err(AppError::ReadCommand)? {
                Command::Quit => break,
                Command::Nop => {}
                Command::Direction(dir) => self.canvas.move_cursor(dir),
                Command::Palette(id) => {
                    let color = *self.palette.color(id);
                    self.canvas.edit(color);
                }
                Command::SetPalette(palette_id, rgb) => {
                    *(self.palette.color_mut(palette_id)) = rgb;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::command::programmed::ProgrammedEvent;
    use crate::command::Direction;
    use crate::widget::palette::PaletteID;

    use super::*;
    #[test]
    fn test_app_run_without_error() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let cs = ProgrammedEvent::new(vec![
            Command::Nop,
            Command::Direction(Direction::Up),
            Command::Direction(Direction::Down),
            Command::Direction(Direction::Left),
            Command::Direction(Direction::Right),
            Command::Palette(PaletteID::ID0),
            Command::Palette(PaletteID::ID1),
            Command::Palette(PaletteID::ID2),
            Command::Palette(PaletteID::ID3),
            Command::Palette(PaletteID::ID4),
            Command::Palette(PaletteID::ID5),
            Command::Quit,
            Command::Nop,
            Command::Nop,
        ]);
        let mut app = App::new(img, cs);
        assert!(matches!(app.run(), Ok(_)));
    }
}
