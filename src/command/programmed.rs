use std::cell::RefCell;

use thiserror::Error;

use super::{Command, CommandStream};

#[derive(Error, Debug)]
pub enum DummyError {}

/// This struct is used to test App automatically.
pub struct ProgrammedEvent {
    remain_commands: RefCell<Vec<Command>>,
}

impl ProgrammedEvent {
    pub fn new(cmds: Vec<Command>) -> Self {
        Self {
            remain_commands: RefCell::new(cmds),
        }
    }
}

impl CommandStream for ProgrammedEvent {
    type Error = DummyError;

    fn read(&self) -> Result<Command, Self::Error> {
        match self.remain_commands.borrow_mut().pop() {
            Some(cmd) => Ok(cmd),
            None => Ok(Command::Unknown),
        }
    }
}
