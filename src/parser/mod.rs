//! Command parsing for lines.
//!
//! Provides facilities for taking an input line and producing a structured result which
//! can more easily be evaluated.
//!
mod geshl;

use std::result;

use super::strings::ShellString;

/// A parser for the geshl language.
///
pub type GeshlParser = geshl::Parser;

/// An error during parsing
///
#[derive(Clone, Debug)]
pub struct Error;

/// Used for results form all parser functions
///
pub type Result<T> = result::Result<T, Error>;

/// A `name=value` variable to set.
///
pub type SetVariable = (String, ShellString);

/// A line that has been parsed
///
#[derive(Clone, Debug, PartialEq)]
pub enum ParsedLine {
    /// Represents an empty parse (a "no op")
    ///
    Empty,

    /// Sets an environment variable.
    ///
    SetVariables(Vec<SetVariable>),

    /// A command to run. This could be either a builtin, alias, function, or command that exists on
    /// the path.
    ///
    Command(Vec<SetVariable>, Vec<ShellString>),
}

/// A parser for shells
pub trait Parser {
    /// Parses `line` into a structured result that can be executed by a shell.
    ///
    fn parse(&self, line: String) -> Result<ParsedLine>;
}
