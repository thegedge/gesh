//! Shell string representation.
//!
//! Strings in a shell can have many components consisting of:
//!
//! - fixed string components,
//! - variable interpolations, and
//! - path components (for example, `~` is the user's home directory).
//!
use std::ops;

/// A string in a shell.
///
#[derive(Clone, Debug, PartialEq)]
pub struct ShellString {
    pieces: Vec<Piece>,
}

/// A component of an interpolated string
///
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    /// A fixed string.
    ///
    Fixed(String),

    /// A glob string.
    ///
    /// Globs can be the following:
    /// - `?`, to match a single character,
    /// - `*`, to match zero or more characters,
    /// - `**`, to match the current directory or arbitrary subdirectories,
    /// - `[...]`, to match any character within the square brackets, or
    /// - `[!...]`, to match any character not within the square brackets.
    ///
    Glob(String),

    /// A shell variable.
    ///
    Variable(String),
}

impl ShellString {
    /// Returns an iterator over the pieces of this `ShellString`.
    ///
    pub fn iter(&self) -> impl Iterator<Item = &Piece> {
        return self.pieces.iter();
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

impl From<Piece> for ShellString {
    fn from(value: Piece) -> Self {
        ShellString { pieces: vec![value] }
    }
}

impl From<Vec<Piece>> for ShellString {
    fn from(value: Vec<Piece>) -> Self {
        ShellString { pieces: value }
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
