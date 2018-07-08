//! A `Parser` implementation for geshl
//!
//! Geshl looks similar to other shell scripting languages, but tries to simplify some
//! aspects of it.
//!
//! TODO: explain how they're different
//!
use nom::{
    self,
    Context,
    InputTakeAtPosition,
    IResult,
    Needed,
};

use parser::{
    self,
    Error,
    ParsedLine,
    Result,
    SetVariable,
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

/// Parses an arbitrary line.
///
named!(
    parse_line(&str) -> ParsedLine,
    alt!(
        command
        | set_variables => { |vars| ParsedLine::SetVariables(vars) }
        | char!('\n') => { |_| ParsedLine::Empty }
    )
);

/// Parses one or more variable setting expressions.
///
/// # Examples
///
/// - `FOO=bar`
/// - `FOO=bar BAR=spam`
///
named!(
    set_variables(&str) -> Vec<SetVariable>,
    sep!(
        space,
        many1!(set_variable)
    )
);

/// Parses a variable setting expression.
///
/// # Examples
///
/// - `FOO=bar`
///
named!(
    set_variable(&str) -> SetVariable,
    do_parse!(
        name: env_var
        >> char!('=')
        >> value: opt!(piece)
        >> ((name.to_owned(), value.unwrap_or_else(|| ShellString::from(""))))
    )
);

/// Parses a command prefixed with zero or more environment variables to set.
///
/// # Examples
///
/// - `FOO=bar command`
/// - `FOO=bar BAR=spam command arg1 "arg2 in quotes"
///
named!(
    command(&str) -> ParsedLine,
    sep!(
        space,
        do_parse!(
            env_vars: many0!(set_variable)
            >> args: many1!(piece)
            >> (ParsedLine::Command(env_vars, args))
        )
    )
);

/// Parses a string "piece".
///
/// A piece could be a path, a glob, an unquoted string, an interpolated string, and so on.
///
named!(
    piece(&str) -> ShellString,
    fold_many1!(
        alt!(
            path
            | glob
            | interpolated_string
            | uninterpolated_string
        ),
        ShellString::from(Vec::new()),
        |acc, string| acc + string
    )
);

/// Parses a glob
///
/// A piece could be a path, a glob, an unquoted string, an interpolated string, and so on.
///
named!(
    glob(&str) -> ShellString,
    map!(
        alt!(
            tag!("?") => { |v| String::from(v) }
            | tag!("**") => { |v| String::from(v) }
            | tag!("*") => { |v| String::from(v) }
            | tag!("[]]") => { |v| String::from(v) }
            | tag!("[!]]") => { |v| String::from(v) }
            | delimited!(char!('['), is_not!("]"), char!(']')) => { |v| format!("[{}]", v) }
        ),
        |v| ShellString::from(Piece::Glob(v))
    )
);

/// Parses a path-like component.
///
/// ## Examples
///
/// - `/`
/// - `/absolute/dir/to/command`
/// - `relative/dir/to/command`
/// - `./current/path`
/// - `../parent/`
/// - `~/home/directory`
///
named!(
    path(&str) -> ShellString,
    map!(
        take_while1!(is_path_character),
        |path| {
            if path.chars().next() == Some('~') {
                // TODO ~foo should reference the home dir of "foo"
                let full_buf = PathBuf::from(path);
                let mut components = full_buf.components();
                components.next();
                let buf = components.as_path().to_path_buf();

                ShellString::from(vec![
                    Piece::Variable("HOME".to_owned()),
                    Piece::from(format!("/{}", OsString::from(buf).to_string_lossy().into_owned())),
                ])
            } else {
                ShellString::from(path)
            }
        }
    )
);

/// Parses an interpolated string from the command line.
///
/// ## Examples
///
/// - `"just some text"`
/// - `"some text with an ${ENVVAR} interpolated"`
///
named!(
    interpolated_string(&str) -> ShellString,
    map!(
        delimited!(
            char!('"'),
            many_till!(alt!(interpolated_env_var | fixed_string), peek!(char!('"'))),
            char!('"')
        ),
        |v| ShellString::from(v.0)
    )
);

/// Parses an environment variable interpolation.
///
/// ## Examples
///
/// - `${HOME}`
/// - `${SOME_DIR}`
///
named!(
    interpolated_env_var(&str) -> Piece,
    map!(
        do_parse!(
            tag!("$")
            >> var: delimited!(tag!("{"), env_var, tag!("}"))
            >> (var)
        ),
        |v| Piece::Variable(v.to_owned())
    )
);

/// Returns whether or not `chr` is valid as a character in a variable name.
///
/// A variable name is composed of alphanumeric characters, and an underscore. If `is_not_first` is
/// `false`, digits are excluded (in other words, the first character in a variable name cannot be a
/// digit).
///
fn env_var(input: &str) -> IResult<&str, &str> {
    let c = input.chars().next().unwrap_or(' ');
    if !is_var_character(c) || c.is_ascii_digit() {
        return Err(nom::Err::Error(Context::Code(input, nom::ErrorKind::IsA)));
    }

    input.split_at_position(|v| !is_var_character(v))
}

/// Returns whether or not `chr` is valid as a character in a variable name.
///
fn is_var_character(chr: char) -> bool {
    match chr {
        'a'...'z' => true,
        'A'...'Z' => true,
        '0'...'9' => true,
        '_' => true,
        _ => false,
    }
}

/// Parses regular text inside of an interpolated string.
///
/// Some characters can be escaped with a backslash.
/// - `\n` becomes a newline
/// - `\r` becomes a carriage return
/// - `\t` becomes a horizontal tab
/// - `\\` becomes a backslash
/// - `\"` becomes a double quote
/// - `\'` becomes a single quote
///
/// ## Examples
///
/// - `abc`
/// - `abc\n123`
/// - `\ttabbed text`
/// - `C:\\Windows\\System\\folder`
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
/// Some characters can be escaped with a backslash:
/// - `\\` becomes a backslash
/// - `\'` becomes a single quote
///
/// ## Examples
///
/// - `'this is just text'`
/// - `'Isn\'t this great?'`
/// - `'This isn't a newline, just a backslash and an n: \n'`
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

/// Split input at space characters, not including newlines / carriage returns
///
fn space(input: &str) -> IResult<&str, &str> {
    input.split_at_position(|c| {
        !c.is_whitespace() || c == '\n' || c == '\r'
    })
}

/// Returns whether or not `chr` is valid as a character in a command name.
///
fn is_path_character(chr: char) -> bool {
    if path::is_separator(chr) {
        return true
    }

    match chr {
        'a'...'z' => true,
        'A'...'Z' => true,
        '0'...'9' => true,
        '~' | '-' | '_' | '.' | '=' => true,
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
     * Tests for `parse_line`
     */
    #[test]
    fn test_parse_line_parses_current_directory() {
        assert_eq!(
            ("\n", ParsedLine::Command(Vec::new(), vec![ShellString::from("./foo.sh")])),
            parse_line("./foo.sh\n").expect("should parse")
        );
    }

    /*
     * Tests for `command`
     */
    #[test]
    fn test_command_parses() {
        assert_eq!(
            ("\n", ParsedLine::Command(
                Vec::new(),
                vec![
                    ShellString::from("/bin/echo"),
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

    #[test]
    fn test_command_parses_with_env_vars() {
        assert_eq!(
            ("\n", ParsedLine::Command(
                vec![
                    ("FOO".to_owned(), ShellString::from("bar"))
                ],
                vec![
                    ShellString::from("export"),
                    ShellString::from("BAR=baz"),
                ]
            )),
            command("FOO=bar export BAR=baz\n").expect("should parse")
        );
    }

    /*
     * Tests for `set_variable` and `set_variables`
     */
    #[test]
    fn test_set_variables_parses_multiple_variables() {
        assert_eq!(
            ("\n", vec![
                ("FOO".to_owned(), ShellString::from("bar")),
                ("BAR".to_owned(), ShellString::from("spam")),
            ]),
            set_variables("FOO=bar BAR=spam\n").expect("should parse")
        );
    }

    #[test]
    fn test_set_variable_parses_variable_without_a_value() {
        assert_eq!(
            ("\n", ("FOO".to_owned(), ShellString::from(""))),
            set_variable("FOO=\n").expect("should parse")
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

    #[test]
    fn test_piece_parses_multiple_adjacent_pieces() {
        assert_eq!(
            ("\n", ShellString::from(vec![
                Piece::from("foo/"),
                Piece::from("bar"),
                Piece::from("/"),
                Piece::Variable("HOME".to_owned()),
                Piece::from(" x"),
                Piece::from("/"),
                Piece::from("more"),
                Piece::from("stuff"),
            ])),
            piece("foo/'bar'/\"${HOME} x\"/'more'\"stuff\"\n").expect("should parse")
        );
    }

    #[test]
    fn test_piece_parses_glob() {
        assert_eq!(
            ("\n", ShellString::from(vec![
                Piece::from("foo/"),
                Piece::from("bar"),
                Piece::from("/"),
                Piece::Variable("HOME".to_owned()),
                Piece::from("/"),
                Piece::Glob("**".to_owned()),
                Piece::from("/"),
                Piece::Glob("*".to_owned()),
                Piece::from(".txt"),
            ])),
            piece("foo/'bar'/\"${HOME}\"/**/*.txt\n").expect("should parse")
        );
    }

    /*
     * Tests for `glob`
     */
    #[test]
    fn test_glob_parses_question_glob() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("?".to_owned()))),
            glob("?\n").expect("should parse")
        );
    }

    #[test]
    fn test_glob_parses_star_glob() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("*".to_owned()))),
            glob("*\n").expect("should parse")
        );
    }

    #[test]
    fn test_glob_parses_recursive_star_glob() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("**".to_owned()))),
            glob("**\n").expect("should parse")
        );
    }

    #[test]
    fn test_glob_parses_character_glob_with_closing_bracket() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("[]]".to_owned()))),
            glob("[]]\n").expect("should parse")
        );
    }

    #[test]
    fn test_glob_parses_character_negation_glob_with_closing_bracket() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("[!]]".to_owned()))),
            glob("[!]]\n").expect("should parse")
        );
    }

    #[test]
    fn test_glob_parses_character_glob() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("[abc]".to_owned()))),
            glob("[abc]\n").expect("should parse")
        );
    }

    #[test]
    fn test_glob_parses_character_negation_glob() {
        assert_eq!(
            ("\n", ShellString::from(Piece::Glob("[!0-9]".to_owned()))),
            glob("[!0-9]\n").expect("should parse")
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

    #[test]
    fn test_path_parses_separator_at_end() {
        assert_eq!(
            ("\n", ShellString::from("/bin/echo/")),
            path("/bin/echo/\n").expect("should parse")
        );
    }

    #[test]
    fn test_path_parses_root_dir() {
        assert_eq!(
            ("\n", ShellString::from("/")),
            path("/\n").expect("should parse")
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
