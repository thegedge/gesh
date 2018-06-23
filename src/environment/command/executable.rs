//! Traits and types for commands, be it an executable on the path or a shell builtin.
//!
use environment::{
    Environment
};

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

/// A trait that represents an arbitrary command.
///
/// Within a shell, a command could be an
/// - executable on the path,
/// - a user-defined alias,
/// - a shell builtin, or
/// - a shell function.
///
/// This trait takes an approach similar to `std::process::Command`, where the executable unit
/// is formed with a builder pattern. All functions that build up the executable unit will
/// return itself so that they can be chained together.
///
/// # Example
///
/// ```
/// SomeExecutableUnit.args(command_args).env(environment).execute()?
/// ```
///
pub trait Command<'e> {
    /// Supply the given arguments as those for this command.
    ///
    fn args(&mut self, args: Vec<String>) -> &mut Self;

    /// Use the given `Environment` for this command.
    ///
    fn env<'v: 'e>(&mut self, env: &'v Environment) -> &mut Self;

    /// Execute this command.
    ///
    fn execute(&mut self) -> Result<ExitStatus, Error>;
}
