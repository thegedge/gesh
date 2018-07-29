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
mod context;
mod path;
mod registry;

pub use self::{
    builtin::*,
    context::Context,
    path::Executable,
    registry::Registry,
};

use std::result;

/// Result type for executing commands.
///
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
#[derive(Debug, PartialEq)]
pub enum ExitStatus {
    /// Instructs the caller that the command requested that the shell exit.
    ///
    ExitWith(u32),

    /// Successfully ran command, with the given status code.
    ///
    Success(u32),
}
