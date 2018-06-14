//! Provides string types that can be interpolated within an environment.
//!
use std::{
    ffi::OsString
};

use super::{
    environment::Environment,
};

/// Enumeration of possible string types that can be supplied on the command line.
///
#[derive(Clone, Debug)]
pub enum ShellString {
    /// A string that contains values that may need to be interpolated within an environment.
    Interpolated(InterpolatedString),

    /// A string without any interpolation needed.
    Uninterpolated(String)
}

/// A component of an interpolated string
#[derive(Clone, Debug)]
pub enum Piece {
    /// A fixed string.
    Fixed(String),

    /// An environment variable.
    Variable(String),
}

impl ShellString {
    /// Converts this shell string into an `OsStr`, interpolating with the given `Environment`.
    ///
    pub fn to_string(&self, env: &Environment) -> OsString {
        match &self {
            ShellString::Interpolated(ref s) => OsString::from(s.interpolate(env)),
            ShellString::Uninterpolated(ref s) => OsString::from(s),
        }
    }
}

impl Piece {
    /// Converts this piece into a `String` with a given environment
    ///
    pub fn to_string(&self, env: &Environment) -> String {
        match &self {
            Piece::Fixed(ref s) => s.clone(),
            Piece::Variable(ref name) => String::from("<todo>"),
        }
    }
}

/// A string that may contain pieces which need to be interpolated within an environment.
#[derive(Clone, Debug)]
pub struct InterpolatedString {
    pieces: Vec<Piece>
}

impl From<Vec<Piece>> for ShellString {
    fn from(pieces: Vec<Piece>) -> Self {
        ShellString::Interpolated(
            InterpolatedString { pieces: pieces }
        )
    }
}

impl InterpolatedString {
    pub fn interpolate(&self, env: &Environment) -> String {
        // TODO estimate string size to avoid unnecessary allocations
        self.pieces.iter().fold(String::new(), |mut s, v| { s.push_str(&v.to_string(env)); s })
    }
}
