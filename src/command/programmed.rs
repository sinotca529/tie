use thiserror::Error;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::widget::Widget;

use super::{Command, CommandStream};

#[derive(Error, Debug)]
pub enum DummyError {}

/// This struct is used to test App automatically.
pub struct ProgrammedEvent {
    remain_commands: Vec<Command>,
}

impl ProgrammedEvent {
    pub fn new(mut cmds: Vec<Command>) -> Self {
        cmds.reverse();
        Self {
            remain_commands: cmds,
        }
    }
}

impl Widget for ProgrammedEvent {
    fn render(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        let cmd_line = Block::default().borders(Borders::ALL);
        let img = Paragraph::new(Text::raw("This is a dummy message"))
            .block(cmd_line)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        f.render_widget(img, rect);
    }
}

impl CommandStream for ProgrammedEvent {
    type Error = DummyError;

    fn read(&mut self) -> Result<Command, Self::Error> {
        match self.remain_commands.pop() {
            Some(cmd) => Ok(cmd),
            None => Ok(Command::Nop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_order() {
        let mut cs = ProgrammedEvent::new(vec![Command::Nop, Command::Quit]);
        assert!(matches!(cs.read(), Ok(Command::Nop)));
        assert!(matches!(cs.read(), Ok(Command::Quit)));
        assert!(matches!(cs.read(), Ok(Command::Nop)));
        assert!(matches!(cs.read(), Ok(Command::Nop)));
    }
}
