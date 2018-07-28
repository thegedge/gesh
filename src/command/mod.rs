//! Executing commands in an environment.
//!
//! Possibilities for commands include:
//!
//! - an executable on the path,
//! - a user-defined alias,
//! - a shell builtin, or
//! - a shell function.
//!
mod builtin;
mod path;
mod registry;

use environment::Environment;

pub use self::{
    builtin::*,
    path::Executable,
    registry::Registry,
};

use std::{
    result
};

/// Result type for executing commands.
pub type Result = result::Result<ExitStatus, Error>;

/// The error type for environment errors.
///
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Command couldn't be found anywhere.
    ///
    UnknownCommand,

    /// Generic error for unknown/uncateogrized errors
    ///
    Unknown,
}

/// Exit status for executing a command.
///
pub enum ExitStatus {
    /// Instructs the caller that the command requested that the shell exit.
    ///
    ExitWith(u32),

    /// Successfully ran command, with the given status code.
    ///
    Success(u32),
}

/// A context for running commands.
///
pub struct Context<'e> {
    pub env: &'e mut Environment,
    pub args: Vec<String>,
}
