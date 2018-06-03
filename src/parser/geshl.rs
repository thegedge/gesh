//! A `Parser` implementation for geshl
//!
//! Geshl looks similar to other shell scripting languages, but tries to simplify some
//! aspects of it.
//!
//! TODO: explain how they're different
//!
use nom::{
    self,
    types::CompleteStr,
};

use parser::{
    self,
    Error,
    ParsedLine,
    Result,
};

/// A parser for geshl
pub struct Parser; 

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
}

impl parser::Parser for Parser {
    /// Parses `line` into a structured result that can be executed by a shell.
    ///
    fn parse<S: AsRef<str>>(&self, line: &S) -> Result<ParsedLine> {
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
