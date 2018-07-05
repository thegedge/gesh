//! Provides string types that can be interpolated within an environment.
//!
use std::{
    ops
};

use super::{
    environment::Environment,
};

/// A string in a shell, which can have many components consisting of:
///
/// - fixed string components,
/// - variable interpolations, and
/// - path components (for example, `~` is the user's home directory).
///
#[derive(Clone, Debug, PartialEq)]
pub struct ShellString {
    pieces: Vec<Piece>,
}

/// A component of an interpolated string
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    /// A fixed string.
    Fixed(String),

    /// A shell variable.
    Variable(String),
}

impl ShellString {
    /// Converts a list of `ShellString`s to a list of `String`s with the given environment.
    ///
    /// Any shell string that cannot be
    ///
    pub fn to_string_vec<V>(values: V, env: &Environment) -> Option<Vec<String>>
        where V: Iterator<Item = ShellString>
    {
        values.map(|a| a.to_string(env))
              .collect::<Option<Vec<_>>>()
    }

    /// Converts this shell string into a regular string.
    ///
    /// Path and variable interpolations are done via the given `Environment`.
    ///
    pub fn to_string(&self, env: &Environment) -> Option<String> {
        // TODO estimate string size to avoid unnecessary allocations
        self.pieces.iter().skip(1).fold(
            self.pieces[0].to_string(env),
            |acc, piece| {
                acc.and_then(|mut v| {
                    let piece_str = piece.to_string(env)?;
                    v.push_str(&piece_str);
                    Some(v)
                })
            }
        )
    }
}

impl ops::Add<ShellString> for ShellString {
    type Output = ShellString;

    fn add(mut self, rhs: ShellString) -> Self::Output {
        self.pieces.extend(rhs.pieces.into_iter());
        ShellString {
            pieces: self.pieces
        }
    }
}

impl <'a> From<&'a str> for ShellString {
    fn from(value: &str) -> Self {
        ShellString { pieces: vec![Piece::Fixed(value.to_owned())] }
    }
}

impl From<String> for ShellString {
    fn from(value: String) -> Self {
        ShellString { pieces: vec![Piece::Fixed(value)] }
    }
}

impl From<Vec<Piece>> for ShellString {
    fn from(value: Vec<Piece>) -> Self {
        ShellString { pieces: value }
    }
}

impl Piece {
    /// Converts this piece into a `String` with a given environment
    ///
    pub fn to_string(&self, env: &Environment) -> Option<String> {
        match &self {
            Piece::Fixed(ref s) => Some(s.clone()),
            Piece::Variable(ref name) => env.get(&name),
        }
    }
}

impl <'a> From<&'a str> for Piece {
    fn from(value: &'a str) -> Self {
        Piece::Fixed(value.to_owned())
    }
}

impl From<String> for Piece {
    fn from(value: String) -> Self {
        Piece::Fixed(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn test_to_string_returns_string_when_var_exists() {
        let shell_string = ShellString::from(vec![
            Piece::from("this is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        let mut vars = HashMap::new();
        vars.insert("WHAT".to_owned(), "test".to_owned());

        let env = Environment::new(vars);

        assert_eq!(Some("this is a test".to_owned()), shell_string.to_string(&env));
    }

    #[test]
    fn test_to_string_returns_none_when_var_doesnt_exist() {
        let shell_string = ShellString::from(vec![
            Piece::from("this is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        let env = Environment::new(HashMap::new());

        assert_eq!(None, shell_string.to_string(&env));
    }

    #[test]
    fn test_to_string_vec_returns_vec_when_all_vars_exist() {
        let shell_strings = vec![
            ShellString::from(vec![
                Piece::from("this is a "),
                Piece::Variable("WHAT".to_owned()),
            ]),
            ShellString::from("another"),
        ];

        let mut vars = HashMap::new();
        vars.insert("WHAT".to_owned(), "test".to_owned());

        let env = Environment::new(vars);

        assert_eq!(
            Some(vec!["this is a test".to_owned(), "another".to_owned()]),
            ShellString::to_string_vec(shell_strings.into_iter(), &env)
        );
    }

    #[test]
    fn test_to_string_vec_returns_none_if_a_var_doesnt_exist() {
        let shell_strings = vec![
            ShellString::from(vec![Piece::Variable("EXISTS".to_owned())]),
            ShellString::from(vec![
                Piece::from("this is a "),
                Piece::Variable("WHAT".to_owned()),
            ]),
            ShellString::from("another"),
        ];

        let mut vars = HashMap::new();
        vars.insert("EXISTS".to_owned(), "i'm here".to_owned());

        let env = Environment::new(vars);

        assert_eq!(None, ShellString::to_string_vec(shell_strings.into_iter(), &env));
    }

    #[test]
    fn test_adding_shellstrings_concatenates() {
        let string1 = ShellString::from("this");
        let string2 = ShellString::from(vec![
            Piece::from(" is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        let expected = ShellString::from(vec![
            Piece::from("this"),
            Piece::from(" is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        assert_eq!(expected, string1 + string2);
    }
}
