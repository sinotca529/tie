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
    pub fn new(mut cmds: Vec<Command>) -> Self {
        cmds.reverse();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_order() {
        let cs = ProgrammedEvent::new(vec![Command::Unknown, Command::Quit]);
        assert!(matches!(cs.read(), Ok(Command::Unknown)));
        assert!(matches!(cs.read(), Ok(Command::Quit)));
        assert!(matches!(cs.read(), Ok(Command::Unknown)));
        assert!(matches!(cs.read(), Ok(Command::Unknown)));
    }
}
