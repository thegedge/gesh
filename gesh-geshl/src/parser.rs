//! A `Parser` implementation for geshl
//!
//! Geshl looks similar to other shell scripting languages, but tries to simplify some
//! aspects of it.
//!
//! TODO: explain how they're different
//!
use nom::{
    self,

    digit1,
    Context,
    InputTakeAtPosition,
    IResult,
    Needed,
};

use std::{
    ffi::OsString,
    path::{
        self,
        PathBuf,
    },
};

use super::{
    Command,
    ParsedLine,
    Piece,
    Redirect,
    RedirectNode,
    RedirectType,
    SetVariable,
    ShellString,
};

/// Parses an arbitrary line.
///
named!(
    pub parse_line(&str) -> ParsedLine,
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
        nom::space1,
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
        >> (SetVariable {
            name: name.to_owned(),
            value: value.unwrap_or_else(|| ShellString::from(""))
        })
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
            vars: many0!(set_variable)
            >> args: many1!(piece)
            >> redirects: many0!(redirect)
            >> (ParsedLine::Command(Command { vars, args, redirects }))
        )
    )
);

/// Parses a string "piece".
///
/// A piece could be a path, a glob, an unquoted string, an interpolated string, and so on.
///
named!(
    piece(&str) -> ShellString,
    terminated!(
        fold_many1!(
            alt!(
                path
                | glob
                | interpolated_string
                | uninterpolated_string
            ),
            ShellString::from(Vec::new()),
            |acc, string| acc + string
        ),
        peek!(not!(char!('<')))
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
            if path.starts_with('~') {
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
        Piece::from
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
        ShellString::from
    )
);

/// Parses an IO redirect.
///
/// IO redirects can take several forms:
///
/// - `>foo.txt` would redirect the command's output to the file `foo.txt`,
/// - `<bar.txt` would redirect `bar.txt` to the stdin of the executed command,
/// - `5>&8` would redirect output to file descriptor 5 into file descriptor 8.
///
named!(
    redirect(&str) -> Redirect,
    map!(
        do_parse!(
            left: redirect_node_left
            >> redirect_type: redirect_type
            >> right: redirect_node_right
            >> (left, redirect_type, right)
        ),
        |(left, redirect_type, right)| {
            match redirect_type {
                RedirectType::In => {
                    Redirect { from: right, to: left, typ: redirect_type }
                },
                RedirectType::OutTruncate | RedirectType::OutAppend => {
                    Redirect { from: left, to: right, typ: redirect_type }
                },
            }
        }
    )
);

/// Parses a redirect type.
///
/// ## Examples
///
/// - `>` redirects output, and truncates the destination
/// - `>>` redirects output, and appends to the destination
/// - `<` redirects input
///
named!(
    redirect_type(&str) -> RedirectType,
    alt!(
        char!('<') => { |_| RedirectType::In }
        | char!('>') => { |_| RedirectType::OutTruncate }
        | tag!(">>") => { |_| RedirectType::OutAppend }
    )
);

/// Parses a redirect node on the left side of a redirect.
///
/// ## Examples
///
/// - `foo.txt`, use a path as a node
/// - `1`, a file descriptor
///
named!(
    redirect_node_left(&str) -> Option<RedirectNode>,
    opt!(
        alt!(
            path => { |p| RedirectNode::File(p) }
            | digit1 => { |n: &str| RedirectNode::Descriptor(n.parse::<i32>().expect("should be an integer")) }
        )
    )
);

/// Parses a redirect node on the left side of a redirect.
///
/// ## Examples
///
/// - `foo.txt`, use a path as a node
/// - `&1`, a file descriptor
///
named!(
    redirect_node_right(&str) -> Option<RedirectNode>,
    opt!(
        alt!(
            path => { |p| RedirectNode::File(p) }
            | preceded!(char!('&'), digit1) => { |n: &str|
                RedirectNode::Descriptor(n.parse::<i32>().expect("should be an integer"))
            }
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    /*
     * Tests for `parse_line`
     */
    #[test]
    fn test_parse_line_parses_current_directory() {
        assert_eq!(
            ("\n", ParsedLine::Command(Command {
                vars: Vec::new(),
                args: vec![ShellString::from("./foo.sh")],
                redirects: vec![],
            })),
            parse_line("./foo.sh\n").expect("should parse")
        );
    }

    /*
     * Tests for `command`
     */
    #[test]
    fn test_command_parses() {
        assert_eq!(
            ("\n", ParsedLine::Command(Command {
                vars: Vec::new(),
                args: vec![
                    ShellString::from("/bin/echo"),
                    ShellString::from("My"),
                    ShellString::from("home"),
                    ShellString::from("dir is"),
                    ShellString::from(vec![
                        Piece::Variable("HOME".to_owned())
                    ])
                ],
                redirects: vec![],
            })),
            command("/bin/echo My home 'dir is' \"${HOME}\"\n").expect("should parse")
        );
    }

    #[test]
    fn test_command_parses_with_env_vars() {
        assert_eq!(
            ("\n", ParsedLine::Command(Command {
                vars: vec![
                    SetVariable { name: "FOO".to_owned(), value: ShellString::from("bar") }
                ],
                args: vec![
                    ShellString::from("export"),
                    ShellString::from("BAR=baz"),
                ],
                redirects: vec![],
            })),
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
                SetVariable { name: "FOO".to_owned(), value: ShellString::from("bar") },
                SetVariable { name: "BAR".to_owned(), value: ShellString::from("spam") },
            ]),
            set_variables("FOO=bar BAR=spam\n").expect("should parse")
        );
    }

    #[test]
    fn test_set_variable_parses_variable_without_a_value() {
        assert_eq!(
            ("\n", SetVariable { name: "FOO".to_owned(), value: ShellString::from("") }),
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

    /*
     * Tests for `redirect`
     */
    #[test]
    fn test_redirect_parses_stdin_redirect_with_file() {
        let from = Some(RedirectNode::File(ShellString::from("foo.txt")));
        let to = None;
        let typ = RedirectType::In;

        assert_eq!(
            ("\n", Redirect { from, to, typ }),
            redirect("<foo.txt\n").expect("should parse")
        );
    }
}
