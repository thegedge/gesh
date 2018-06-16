//! Provides string types that can be interpolated within an environment.
//!
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

    /// An environment variable.
    Variable(String),
}

impl ShellString {
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

impl <'a> From<&'a str> for ShellString {
    fn from(value: &'a str) -> Self {
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
            Piece::Variable(ref name) => env.var(&name),
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
