use crate::{
    command::{Command, Direction},
    widget::palette::PaletteID,
};
use crossterm::event::KeyCode;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KeyConfig {
    config: HashMap<KeyCode, Command>,
    cmd_line_prefix: char,
    palette_id2char: [char; PaletteID::NUM_COLORS],
}

impl KeyConfig {
    fn palette_id2char(&self, id: PaletteID) -> char {
        self.palette_id2char[id as usize]
    }

    pub fn char2palette_id(&self, ch: char) -> Option<PaletteID> {
        use PaletteID::*;
        for id in [ID0, ID1, ID2, ID3, ID4, ID5] {
            if self.palette_id2char(id) == ch {
                return Some(id);
            }
        }
        None
    }

    pub fn get(&self, key: &KeyCode) -> Option<&Command> {
        self.config.get(key)
    }

    pub fn cmd_line_prefix(&self) -> char {
        self.cmd_line_prefix
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            config: [
                (KeyCode::Char('q'), Command::Quit),
                (KeyCode::Char('h'), Command::Direction(Direction::Left)),
                (KeyCode::Char('j'), Command::Direction(Direction::Down)),
                (KeyCode::Char('k'), Command::Direction(Direction::Up)),
                (KeyCode::Char('l'), Command::Direction(Direction::Right)),
                (KeyCode::Char('w'), Command::Palette(PaletteID::ID0)),
                (KeyCode::Char('e'), Command::Palette(PaletteID::ID1)),
                (KeyCode::Char('r'), Command::Palette(PaletteID::ID2)),
                (KeyCode::Char('s'), Command::Palette(PaletteID::ID3)),
                (KeyCode::Char('d'), Command::Palette(PaletteID::ID4)),
                (KeyCode::Char('f'), Command::Palette(PaletteID::ID5)),
            ]
            .into_iter()
            .collect(),
            cmd_line_prefix: ':',
            palette_id2char: ['w', 'e', 'r', 's', 'd', 'f'],
        }
    }
}
