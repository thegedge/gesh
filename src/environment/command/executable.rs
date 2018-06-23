//! Traits and types for executable "units".
//!
use environment::{
    Environment
};

/// The error type for environment errors.
///
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Command wasn't found on the path
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

/// A trait that represents an executable "unit".
///
/// Within a shell, an executable unit could be a
/// - command,
/// - alias,
/// - builtin, or
/// - function.
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
pub trait ExecutableUnit<'e> {
    /// Supply the given arguments as those for this executable unit.
    ///
    fn args(&mut self, args: Vec<String>) -> &mut Self;

    /// Use the given `Environment` for this executable.
    ///
    /// TODO: May need a mutable ref of some type, for things like `export` that would mutate `env`
    ///
    fn env<'v: 'e>(&mut self, env: &'v Environment) -> &mut Self;

    ///
    ///
    fn execute(&mut self) -> Result<ExitStatus, Error>;
}
