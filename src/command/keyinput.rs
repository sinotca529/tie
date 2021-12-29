use crossterm::event::{self, KeyCode};

use super::{Command, CommandStream, Direction};

/// Fetch key event and use it as Command
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct KeyInput;

impl KeyInput {
    fn keycode2command(keycode: &KeyCode) -> Command {
        match keycode {
            KeyCode::Char('q') => Command::Quit,
            KeyCode::Char('h') => Command::Direction(Direction::Left),
            KeyCode::Char('j') => Command::Direction(Direction::Down),
            KeyCode::Char('k') => Command::Direction(Direction::Up),
            KeyCode::Char('l') => Command::Direction(Direction::Right),
            _ => Command::Unknown,
        }
    }
}

impl CommandStream for KeyInput {
    type Error = std::io::Error;

    fn read(&self) -> Result<Command, Self::Error> {
        event::read().map(|op| match op {
            event::Event::Key(key) => Self::keycode2command(&key.code),
            _ => Command::Unknown,
        })
    }
}
