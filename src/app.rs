use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use crate::{
    command::{Command, CommandStream},
    image::Image,
    widget::{
        canvas::{self, Canvas},
        palette::Palette,
        Widget,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum Error<E: 'static + std::error::Error + std::fmt::Debug> {
    #[error("IO error in terminal initialization.")]
    InitTerm(#[source] std::io::Error),
    #[error("IO error in terminal finalization.")]
    FinTerm(#[source] std::io::Error),
    #[error("IO error in rendering process.")]
    Render(#[source] std::io::Error),
    #[error("Error in read command.")]
    ReadCommand(#[source] E),
    #[error("Error in canvas.")]
    Canvas(#[source] canvas::Error),
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
    pub fn run(&mut self) -> Result<(), Error<CS::Error>> {
        // Setup terminal
        enable_raw_mode().map_err(Error::InitTerm)?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(Error::InitTerm)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(Error::InitTerm)?;

        // create app and run it
        self.main_loop(&mut terminal)?;

        // restore terminal
        disable_raw_mode().map_err(Error::FinTerm)?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen,).map_err(Error::FinTerm)?;
        terminal.show_cursor().map_err(Error::FinTerm)?;

        Ok(())
    }

    fn main_loop(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), Error<CS::Error>> {
        loop {
            terminal.draw(|f| self.render(f)).map_err(Error::Render)?;

            match self.cmd_stream.read().map_err(Error::ReadCommand)? {
                Command::Quit => break,
                Command::Nop => {}
                Command::Direction(dir) => self.canvas.move_cursor(dir),
                Command::Palette(id) => {
                    let color = *self.palette.color(id);
                    self.canvas.paint(color);
                }
                Command::SetPalette(palette_id, rgb) => {
                    self.palette.set_color(palette_id, rgb);
                }
                Command::Save => self.canvas.save().map_err(Error::Canvas)?,
                Command::SaveAs(path) => self.canvas.save_as(path).map_err(Error::Canvas)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::command::programmed::ProgrammedEvent;
    use crate::command::Direction;
    use crate::image::Rgb;
    use crate::widget::palette::PaletteCellId;

    use super::*;
    #[test]
    fn test_app_run_without_error() {
        let tmp_path1 = "tests/image/app_test_app_run_without_error1.png";
        let tmp_path2 = "tests/image/app_test_app_run_without_error2.png";
        std::fs::copy("tests/image/00.png", tmp_path1).unwrap();

        let img = Image::open(tmp_path1).unwrap();
        let cs = ProgrammedEvent::new(vec![
            Command::Nop,
            Command::Direction(Direction::Up),
            Command::Direction(Direction::Down),
            Command::Direction(Direction::Left),
            Command::Direction(Direction::Right),
            Command::Palette(PaletteCellId::Id0),
            Command::Palette(PaletteCellId::Id1),
            Command::Palette(PaletteCellId::Id2),
            Command::Palette(PaletteCellId::Id3),
            Command::Palette(PaletteCellId::Id4),
            Command::Palette(PaletteCellId::Id5),
            Command::SetPalette(PaletteCellId::Id0, Rgb(0, 0, 0)),
            Command::Save,
            Command::SaveAs(tmp_path2.into()),
            Command::Quit,
            Command::Nop,
            Command::Nop,
        ]);
        let mut app = App::new(img, cs);
        assert!(matches!(app.run(), Ok(_)));

        std::fs::remove_file(tmp_path1).unwrap();
        std::fs::remove_file(tmp_path2).unwrap();
    }
}
