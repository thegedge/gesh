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
    ParsedLine,
    Parser,
};

pub struct Shell<R: Reader> {
    pub reader: R,
    pub parser: Parser,
}

/// Enumeration of all possible errors that can occur in the shell
///
#[derive(Debug)]
pub enum Error {
    ReadlineError(readline::Error),
}

impl<R: Reader> Shell<R> {
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.reader.get() {
                Ok(raw_line) => self.parse_line(raw_line),
                Err(readline::Error::Eof()) => break,
                Err(err) => return Err(Error::ReadlineError(err)),
            }
        }
        Ok(())
    }

    fn parse_line(&mut self, raw_line: String) {
        match self.parser.parse(&raw_line) {
            Ok(parsed_line) => match parsed_line {
                ParsedLine::Command(cmd) => println!("command: {:?}", cmd),
            },
            Err(err) => {
                println!("error: {:?}", err)
            }
        }
    }
}
