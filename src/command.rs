use crate::widget::palette::PaletteID;

pub mod keyinput;

#[cfg(test)]
pub mod programmed;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Command {
    Quit,
    Unknown,
    Direction(Direction),
    Palette(PaletteID),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub trait CommandStream {
    type Error;
    /// This function blocks until a command is available.
    fn read(&self) -> Result<Command, Self::Error>;
}
