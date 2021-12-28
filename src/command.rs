pub mod keyinput;

#[cfg(test)]
pub mod programmed;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Command {
    Quit,
    Unknown,
}

pub trait CommandStream {
    type Error;
    /// This function blocks until a command is available.
    fn read(&self) -> Result<Command, Self::Error>;
}
