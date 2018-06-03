//! Encapsulates the core components of a shell.
//!
//! This inclues:
//! * Input
//! * Command parsing
//! * Job management
//!
use readline::{
    self,
    Reader,
};

use parser::{
    self,
    ParsedLine,
    Parser,
};

/// A user shell.
///
pub struct Shell<R: Reader, P: Parser> {
    pub reader: R,
    pub parser: P,
}

/// Enumeration of all possible errors that can occur in the shell.
///
#[derive(Debug)]
pub enum Error {
    ReadlineError(readline::Error),
    ParserError(parser::Error),
}

impl<R: Reader, P: Parser> Shell<R, P> {
    /// Runs the shell's main read -> parse -> execute loop.
    ///
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let parsed_line = match self.reader.get() {
                Ok(raw_line) => self.parser.parse(&raw_line)?,
                Err(readline::Error::Eof()) => break,
                Err(readline::Error::Interrupted()) => continue,
                Err(err) => return Err(Error::ReadlineError(err)),
            };

            match parsed_line {
                ParsedLine::Command(cmd) => println!("command: {:?}", cmd),
                ParsedLine::Empty => continue,
            }
        }
        Ok(())
    }
}

impl From<readline::Error> for Error {
    fn from(err: readline::Error) -> Self {
        Error::ReadlineError(err)
    }
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Self {
        Error::ParserError(err)
    }
}
