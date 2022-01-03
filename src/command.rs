use crate::{
    image::Rgb,
    widget::{palette::PaletteID, Widget},
};

pub mod keyinput;

#[cfg(test)]
pub mod programmed;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Command {
    Quit,
    Nop,
    Direction(Direction),
    Palette(PaletteID),
    SetPalette(PaletteID, Rgb),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub trait CommandStream: Widget {
    type Error;
    /// This function blocks until a command is available.
    fn read(&mut self) -> Result<Command, Self::Error>;
}
