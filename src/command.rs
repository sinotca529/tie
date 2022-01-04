use std::path::PathBuf;

use crate::{
    image::Rgb,
    widget::{palette::PaletteCellID, Widget},
};

pub mod keyinput;

#[cfg(test)]
pub mod programmed;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Command {
    Quit,
    Nop,
    Direction(Direction),
    Palette(PaletteCellID),
    SetPalette(PaletteCellID, Rgb),
    Save,
    SaveAs(PathBuf),
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

    /// Read a Command.
    /// This function blocks until a command is available.
    fn read(&mut self) -> Result<Command, Self::Error>;
}
