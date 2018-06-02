//! Encapsulates the core components of a shell.
//!
//! This inclues:
//! * Input
//! * Command parsing
//! * Job management
use readline::{
    self,
    Reader,
};

pub struct Shell<R: Reader> {
    pub reader: R,
}

/// Enumeration of all possible errors that can occur in the shell
#[derive(Debug)]
pub enum Error {
    ReadlineError(readline::Error),
}

impl<R: Reader> Shell<R> {
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.reader.get() {
                Ok(command) => {
                    println!("command: {}", command)
                },
                Err(readline::Error::Eof()) => break,
                Err(err) => return Err(Error::ReadlineError(err)),
            }
        }
        Ok(())
    }
}
