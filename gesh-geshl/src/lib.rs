//! Command parsing for lines.
//!
//! Provides facilities for taking an input line and producing a structured result which
//! can more easily be evaluated.
//!
#[macro_use] extern crate nom;

mod parser;
mod strings;

use std::result;

pub use strings::{
    Piece,
    ShellString,
};

/// An error during parsing
///
#[derive(Clone, Debug)]
pub struct Error;

/// Used for results form all parser functions
///
pub type Result<T> = result::Result<T, Error>;

/// A `name=value` variable to set.
///
#[derive(Clone, Debug, PartialEq)]
pub struct SetVariable {
    pub name: String,
    pub value: ShellString,
}

/// A node in a redirect (source/sink)
///
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectNode {
    /// Redirect to/from a file path
    File(ShellString),

    /// Redirect to a file descriptor
    Descriptor(i32),
}

/// The type of redirect.
///
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectType {
    /// Redirect output, append to the output location
    OutAppend,

    /// Redirect output, truncate the output location
    OutTruncate,

    /// Redirect input from one file to another
    In,
}

/// An IO redirect.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Redirect {
    from: Option<RedirectNode>,
    to: Option<RedirectNode>,
    typ: RedirectType
}

/// A command and its context.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub vars: Vec<SetVariable>,
    pub args: Vec<ShellString>,
    pub redirects: Vec<Redirect>,
}

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
    Command(Command),
}

/// A parser for geshl.
///
pub struct Parser;

impl Parser {
    /// Constructs a new `Parser`
    ///
    pub fn new() -> Parser {
        Parser
    }

    /// Parses `line` into a structured result that can be executed by a shell.
    ///
    pub fn parse(&self, mut line: String) -> Result<ParsedLine> {
        line.push('\n');

        let parse_result = parser::parse_line(&line);
        println!("{:?}", parse_result);
        match parse_result {
            Ok((_, parsed_line)) => Ok(parsed_line),
            Err(nom::Err::Incomplete(_)) => Err(Error),
            Err(nom::Err::Error(_)) => Err(Error),
            Err(nom::Err::Failure(_)) => Err(Error),
        }
    }
}
