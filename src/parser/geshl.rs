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

use std::{
    ffi::OsString,
    path::{
        self,
        PathBuf,
    },
};

use strings::{
    Piece,
    ShellString,
};

/// A parser for geshl.
///
pub struct Parser; 

/// Parses a command and its arguments.
///
named!(
    command(&str) -> ParsedLine,
    sep!(
        space,
        do_parse!(
            command: piece
            >> args: many0!(piece)
            >> (ParsedLine::Command(command, args))
        )
    )
);

/// Parses a string "piece".
///
/// A piece could be a path, an unquoted string, an interpolated string, and so on.
///
named!(
    piece(&str) -> ShellString,
    alt!(
        path
        | interpolated_string
        | uninterpolated_string
    )
);

/// Parses a path-like component.
///
named!(
    path(&str) -> ShellString,
    map!(
        pair!(
            opt!(take_while1!(path::is_separator)),
            separated_nonempty_list!(
                take_while1!(path::is_separator),
                take_while1!(is_command_character)
            )
        ),
        |(absolute, components)| {
            // TODO Try to simplify this, maybe with an alt! parser
            let mut buf = PathBuf::new();
            let mut pieces = Vec::new();

            if absolute.is_some() {
                buf.push(path::MAIN_SEPARATOR.to_string());
                components.iter().for_each(|piece| buf.push(piece));
            } else if components.len() > 0 {
                if components[0] == "~" {
                    pieces.push(Piece::Variable("HOME".to_owned()));
                    buf.push(path::MAIN_SEPARATOR.to_string());
                    components.iter().skip(1).for_each(|piece| buf.push(piece));
                } else {
                    components.iter().for_each(|piece| buf.push(piece));
                }
            }

            // TODO maybe consider an enum for shell strings so we don't have to do this conversion
            pieces.push(Piece::from(OsString::from(buf).to_string_lossy().into_owned()));
            ShellString::from(pieces)
        }
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
        |v| Piece::from(v)
    )
);

/// Parses an uninterpolated string from the command line.
///
named!(
    uninterpolated_string(&str) -> ShellString,
    map!(
        delimited!(
            char!('\''),
            escaped_transform!(
                none_of!("'\\"),
                '\\',
                alt!(
                    char!('\\') => { |_| "\\" }
                    | char!('\'') => { |_| "'" }
                )
            ),
            char!('\'')
        ),
        |v| ShellString::from(v)
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

/// Split input at space characters, not including newlines / carriage returns
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

/// Returns whether or not `chr` is valid as a character in a command name.
///
fn is_command_character(chr: char) -> bool {
    match chr {
        'a'...'z' => true,
        'A'...'Z' => true,
        '0'...'9' => true,
        '~' | '-' | '_' | '.' => true,
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

#[cfg(test)]
mod tests {
    use super::*;

    /*
     * Tests for `command`
     */
    #[test]
    fn test_command_parses() {
        assert_eq!(
            ("\n", ParsedLine::Command(
                ShellString::from("/bin/echo"),
                vec![
                    ShellString::from("My"),
                    ShellString::from("home"),
                    ShellString::from("dir is"),
                    ShellString::from(vec![
                        Piece::Variable("HOME".to_owned())
                    ])
                ]
            )),
            command("/bin/echo My home 'dir is' \"${HOME}\"\n").expect("should parse")
        );
    }

    /*
     * Tests for `piece`
     */
    #[test]
    fn test_piece_parses_paths() {
        assert_eq!(
            ("\n", ShellString::from("/bin/echo")),
            piece("/bin/echo\n").expect("should parse")
        );
    }

    #[test]
    fn test_piece_parses_interpolated_strings() {
        assert_eq!(
            ("\n", ShellString::from(vec![
                Piece::from("echo"),
            ])),
            piece("\"echo\"\n").expect("should parse")
        );
    }

    #[test]
    fn test_piece_parses_uninterpolated_strings() {
        assert_eq!(
            ("\n", ShellString::from("echo")),
            piece("'echo'\n").expect("should parse")
        );
    }

    /*
     * Tests for `path`
     */
    #[test]
    fn test_path_parses_absolute_paths() {
        assert_eq!(
            ("\n", ShellString::from("/bin/echo")),
            path("/bin/echo\n").expect("should parse")
        );
    }

    #[test]
    fn test_path_parses_relative_paths() {
        assert_eq!(
            ("\n", ShellString::from("bin/echo")),
            path("bin/echo\n").expect("should parse")
        );
    }

    #[test]
    fn test_path_parses_tilde_paths() {
        assert_eq!(
            ("\n", ShellString::from(vec![
                Piece::Variable("HOME".to_owned()),
                Piece::from("/bin/echo")
            ])),
            path("~/bin/echo\n").expect("should parse")
        );
    }

    /*
     * Tests for `interpolated_string`
     */
    #[test]
    fn test_interpolated_string_parses_simple_string() {
        assert_eq!(
            ("", ShellString::from(vec![
                Piece::from("this is a test"),
            ])),
            interpolated_string("\"this is a test\"").expect("should parse")
        );
    }

    #[test]
    fn test_interpolated_string_parses_complex_string() {
        assert_eq!(
            ("", ShellString::from(vec![
                Piece::from("home dir:\n\t"),
                Piece::Variable("HOME".to_owned()),
                Piece::from("\n\ncode dir:\n\t"),
                Piece::Variable("CODE_DIR".to_owned()),
            ])),
            interpolated_string("\"home dir:\\n\\t${HOME}\\n\\ncode dir:\\n\\t${CODE_DIR}\"").expect("should parse")
        );
    }

    #[test]
    fn test_interpolated_string_parses_string_escapes() {
        assert_eq!(
            ("", ShellString::from(vec![
                Piece::from("\tthis is \n a \"test\"")
            ])),
            interpolated_string("\"\\tthis is \\n a \\\"test\\\"\"").expect("should parse")
        );
    }

    #[test]
    fn test_interpolated_string_parses_string_with_vars() {
        assert_eq!(
            ("", ShellString::from(vec![
                Piece::from("\tthis is \n a \"test\"")
            ])),
            interpolated_string("\"\\tthis is \\n a \\\"test\\\"\"").expect("should parse")
        );
    }

    /*
     * Tests for `uninterpolated_string`
     */
    #[test]
    fn test_uninterpolated_string_parses_simple_string() {
        assert_eq!(
            ("", ShellString::from("this is a test")),
            uninterpolated_string("'this is a test'").expect("should parse")
        );
    }

    #[test]
    fn test_uninterpolated_string_parses_string_with_escapes() {
        assert_eq!(
            ("", ShellString::from("it's \\ a test")),
            uninterpolated_string("'it\\'s \\\\ a test'").expect("should parse")
        );
    }
}
