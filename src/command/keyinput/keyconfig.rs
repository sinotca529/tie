use crate::{
    command::{Command, Direction},
    widget::palette::{Palette, PaletteCellID},
};
use crossterm::event::KeyCode;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KeyConfig {
    config: HashMap<KeyCode, Command>,
    palette_id2char: [char; Palette::NUM_CELL],
}

impl KeyConfig {
    fn palette_cell_id2char(&self, id: PaletteCellID) -> char {
        self.palette_id2char[id as usize]
    }

    pub fn char2palette_cell_id(&self, ch: char) -> Option<PaletteCellID> {
        use PaletteCellID::*;
        for id in [ID0, ID1, ID2, ID3, ID4, ID5] {
            if self.palette_cell_id2char(id) == ch {
                return Some(id);
            }
        }
        None
    }

    pub fn get(&self, key: &KeyCode) -> Option<&Command> {
        self.config.get(key)
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            config: [
                (KeyCode::Char('h'), Command::Direction(Direction::Left)),
                (KeyCode::Char('j'), Command::Direction(Direction::Down)),
                (KeyCode::Char('k'), Command::Direction(Direction::Up)),
                (KeyCode::Char('l'), Command::Direction(Direction::Right)),
                (KeyCode::Char('w'), Command::Palette(PaletteCellID::ID0)),
                (KeyCode::Char('e'), Command::Palette(PaletteCellID::ID1)),
                (KeyCode::Char('r'), Command::Palette(PaletteCellID::ID2)),
                (KeyCode::Char('s'), Command::Palette(PaletteCellID::ID3)),
                (KeyCode::Char('d'), Command::Palette(PaletteCellID::ID4)),
                (KeyCode::Char('f'), Command::Palette(PaletteCellID::ID5)),
            ]
            .into_iter()
            .collect(),
            palette_id2char: ['w', 'e', 'r', 's', 'd', 'f'],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_palette_id2char() {
        let kc = KeyConfig::default();
        assert_eq!(kc.palette_cell_id2char(PaletteCellID::ID0), 'w');
        assert_eq!(kc.palette_cell_id2char(PaletteCellID::ID1), 'e');
    }

    #[test]
    fn test_char2palette_id() {
        let kc = KeyConfig::default();
        assert_eq!(kc.char2palette_cell_id('w'), Some(PaletteCellID::ID0));
        assert_eq!(kc.char2palette_cell_id('e'), Some(PaletteCellID::ID1));
        assert_eq!(kc.char2palette_cell_id('W'), None);
    }

    #[test]
    fn test_get() {
        let kc = KeyConfig::default();
        assert_eq!(
            kc.get(&KeyCode::Char('h')),
            Some(&Command::Direction(Direction::Left))
        );
        assert_eq!(
            kc.get(&KeyCode::Char('r')),
            Some(&Command::Palette(PaletteCellID::ID2))
        );
        assert_eq!(kc.get(&KeyCode::Char('!')), None);
    }
}
