//! Provides string types that can be interpolated within an environment.
//!
use std::{
    env::VarError,
};

use super::{
    environment::Environment,
};

/// Enumeration of possible string types that can be supplied on the command line.
///
#[derive(Clone, Debug, PartialEq)]
pub enum ShellString {
    /// A string that contains values that may need to be interpolated within an environment.
    Interpolated(InterpolatedString),

    /// A string without any interpolation needed.
    Uninterpolated(String)
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
    /// Converts this shell string into an `OsStr`, interpolating with the given `Environment`.
    ///
    pub fn to_string(&self, env: &Environment) -> Result<String, VarError> {
        match &self {
            ShellString::Interpolated(ref s) => s.interpolate(env),
            ShellString::Uninterpolated(ref s) => Ok(s.clone()),
        }
    }
}

impl Piece {
    /// Converts this piece into a `String` with a given environment
    ///
    pub fn to_string(&self, env: &Environment) -> Result<String, VarError> {
        match &self {
            Piece::Fixed(ref s) => Ok(s.clone()),
            Piece::Variable(ref name) => match env.var(name) {
                Ok(v) => Ok(v),
                Err(VarError::NotPresent) => Ok(String::new()),
                Err(e) => Err(e),
            }
        }
    }
}

/// A string that may contain pieces which need to be interpolated within an environment.
#[derive(Clone, Debug, PartialEq)]
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
    pub fn interpolate(&self, env: &Environment) -> Result<String, VarError> {
        // TODO estimate string size to avoid unnecessary allocations
        self.pieces.iter().skip(1).fold(
            self.pieces[0].to_string(env),
            |acc, piece| {
                acc.and_then(|mut v| {
                    let piece_str = piece.to_string(env)?;
                    v.push_str(&piece_str);
                    Ok(v)
                })
            }
        )
    }
}
