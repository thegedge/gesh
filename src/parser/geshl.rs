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
    AsChar,
    FindToken,
    InputTakeAtPosition,
    IResult,
};

use parser::{
    self,
    Error,
    ParsedLine,
    Result,
};


/// A parser for geshl.
///
pub struct Parser; 

/// Space-separated parsing, which doesn't included newlines/carriage returns
///
fn space<'a, T>(input: T) -> IResult<T, T>
where
  T: InputTakeAtPosition,
  <T as InputTakeAtPosition>::Item: AsChar + Clone,
  &'a str: FindToken<<T as InputTakeAtPosition>::Item>,
{
  input.split_at_position(|item| {
    let c = item.clone().as_char();
    !c.is_whitespace() || c == '\n' || c == '\r'
  })
}

macro_rules! spaced (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::{Convert, Err};

      let (i1, o) = sep!($i, space, $($args)*)?;
      match space(i1) {
        Err(e) => Err(Err::convert(e)),
        Ok((i2, _)) => Ok((i2, o))
      }
    }
  )
);

/// Parses a command and its arguments.
///
named!(
    command(CompleteStr) -> ParsedLine,
    map!(
        spaced!(
            do_parse!(
                command: argument >>
                args: many0!(argument) >>
                (command, args)
            )
        ),
        |(command, args)| {
            ParsedLine::Command(
                String::from(command.as_ref()),
                Vec::from(args),
            )
        }
    )
);

/// Parses an "argument" from the command line.
///
named!(
    argument(CompleteStr) -> String,
    map!(
        alt!(
            take_while1!(is_command_character) |
            interpolated_string |
            uninterpolated_string
        ),
        |v| String::from(v.as_ref())
    )
);

/// Parses an interpolated string from the command line.
///
named!(
    interpolated_string(CompleteStr) -> CompleteStr,
    delimited!(
        tag!("\""),
        take_until!("\""),
        tag!("\"")
    )
);

/// Parses an uninterpolated string from the command line.
///
named!(
    uninterpolated_string(CompleteStr) -> CompleteStr,
    delimited!(
        tag!("'"),
        take_until!("'"),
        tag!("'")
    )
);

/// Parses an arbitrary line.
///
named!(
    parse_line(CompleteStr) -> ParsedLine,
    ws!(
        alt!(
            command
            | map!(eof!(), |_v| ParsedLine::Empty)
        )
    )
);

/// Returns whether or not `chr` is valid as a character in a command name.
///
fn is_command_character(chr: char) -> bool {
    match chr {
        'a'...'z' => true,
        'A'...'Z' => true,
        '0'...'9' => true,
        '-' | '_' | '/' | '.' => true,
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
