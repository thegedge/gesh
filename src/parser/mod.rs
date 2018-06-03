//! Command parsing for lines.
//!
//! Provides facilities for taking an input line and producing a structured result which
//! can more easily be evaluated.
//!
mod geshl;

use std::result;

pub type GeshlParser = geshl::Parser;

/// An error during parsing
#[derive(Clone, Debug)]
pub struct Error;

/// Uses for results form all parser functions
pub type Result<T> = result::Result<T, Error>;

/// A line that has been parsed
#[derive(Clone, Debug)]
pub enum ParsedLine {
    /// Represents an empty parse (a "no op")
    Empty,

    /// The name of a command. This could be either a builtin, alias, function, or command
    /// that exists on the path
    Command(String)
}

/// A parser for shells
pub trait Parser {
    /// Parses `line` into a structured result that can be executed by a shell.
    ///
    fn parse<S: AsRef<str>>(&self, line: &S) -> Result<ParsedLine>;
}
