//! A `Parser` implementation for geshl
//!
//! Geshl looks similar to other shell scripting languages, but tries to simplify some
//! aspects of it.
//!
//! TODO: explain how they're different
//!
use nom::{
    self,
    AsChar,
    FindToken,
    InputTakeAtPosition,
    IResult,
    Needed,
};

use parser::{
    self,
    Error,
    ParsedLine,
    Result,
};

use strings::{
    Piece,
    ShellString,
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
    command(&str) -> ParsedLine,
    spaced!(
        do_parse!(
            command: argument
            >> args: many0!(argument)
            >> (ParsedLine::Command(command, args))
        )
    )
);

/// Parses an "argument" from the command line.
///
named!(
    argument(&str) -> ShellString,
    alt!(
        take_while1!(is_command_character) => { |v: &str| ShellString::Uninterpolated(v.to_owned()) }
        | interpolated_string
        | uninterpolated_string
    )
);

/// Parses an interpolated string from the command line.
///
named!(
    interpolated_string(&str) -> ShellString,
    map!(
        delimited!(
            char!('"'),
            many_till!(alt!(env_var | fixed_string), peek!(char!('"'))),
            char!('"')
        ),
        |v| ShellString::from(v.0)
    )
);

/// Parses an environment variable.
///
named!(
    env_var(&str) -> Piece,
    map!(
        do_parse!(
            tag!("$")
            >> var: delimited!(tag!("{"), take_until!("}"), tag!("}"))
            >> (var)
        ),
        |v| Piece::Variable(v.to_owned())
    )
);

/// Parses regular text inside of an interpolated string.
///
named!(
    fixed_string(&str) -> Piece,
    map!(
        escaped_transform!(
            none_of!("$\"\\"),
            '\\',
            alt!(
                tag!("\\") => { |_| "\\" }
                | tag!("\"") => { |_| "\"" }
                | tag!("\'") => { |_| "'" }
                | tag!("n") => { |_| "\n" }
                | tag!("r") => { |_| "\r" }
                | tag!("t") => { |_| "\t" }
            )
        ),
        |v| Piece::Fixed(v.to_owned())
    )
);

/// Parses an uninterpolated string from the command line.
///
named!(
    uninterpolated_string(&str) -> ShellString,
    map!(
        delimited!(
            tag!("'"),
            escaped_transform!(
                none_of!("'\\"),
                '\\',
                alt!(
                    tag!("\\") => { |_| "\\" }
                    | tag!("\'") => { |_| "'" }
                )
            ),
            tag!("'")
        ),
        |v| ShellString::Uninterpolated(v.to_owned())
    )
);

/// Parses an arbitrary line.
///
named!(
    parse_line(&str) -> ParsedLine,
    alt!(
        command
        | char!('\n') => { |_| ParsedLine::Empty }
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
    fn parse(&self, mut line: String) -> Result<ParsedLine> {
        line.push('\n');

        let parse_result = parse_line(&line);
        match parse_result {
            Ok((_, parsed_line)) => Ok(parsed_line),
            Err(nom::Err::Incomplete(_)) => Err(Error),
            Err(nom::Err::Error(_)) => Err(Error),
            Err(nom::Err::Failure(_)) => Err(Error),
        }
    }
}
