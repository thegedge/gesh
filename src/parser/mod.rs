//! Command parsing for lines.
//!
//! Provides facilities for taking an input line and producing a structured result which
//! can more easily be evaluated.
//!
use nom::{
    self,
    types::CompleteStr,
};
use std::result;

/// A parser for shells
pub struct Parser; 

/// A line that has been parsed
#[derive(Clone, Debug)]
pub enum ParsedLine {
    /// Represents an empty parse (a "no op")
    Empty,

    /// The name of a command. This could be either a builtin, alias, function, or command
    /// that exists on the path
    Command(String)
}

/// An error during parsing
#[derive(Clone, Debug)]
pub struct Error;

/// Uses for results form all parser functions
type Result<T> = result::Result<T, Error>;

/// Parses a command
named!(
    command(CompleteStr) -> ParsedLine,
    map!(
        take_while1!(is_command_character),
        |v| ParsedLine::Command(String::from(v.as_ref()))
    )
);

/// Parses an arbitrary line
named!(
    parse_line(CompleteStr) -> ParsedLine,
    ws!(
        alt!(
            command
            | map!(eof!(), |_v| ParsedLine::Empty)
        )
    )
);

/// Returns `true` if `chr` is valid as a character in a command name.
///
/// Return `false` otherwise.
///
fn is_command_character(chr: char) -> bool {
    match chr {
        'a'...'z' => true,
        'A'...'Z' => true,
        '0'...'9' => true,
        '-' | '_' => true,
        _ => false,
    }
}

impl Parser {
    /// Constructs a new `Parser`
    ///
    pub fn new() -> Parser {
        Parser
    }

    /// Parses `line` into a structured result that can be executed by a shell.
    ///
    pub fn parse<S: AsRef<str>>(&self, line: &S) -> Result<ParsedLine> {
        let parse_result = parse_line(CompleteStr(line.as_ref()));
        println!("{:?}", parse_result);

        match parse_result {
            Ok((_, parsed_line)) => Ok(parsed_line),
            Err(nom::Err::Incomplete(_)) => Err(Error),
            Err(nom::Err::Error(_)) => Err(Error),
            Err(nom::Err::Failure(_)) => Err(Error),
        }
    }
}
