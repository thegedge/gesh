//! Executing commands in an environment.
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

use std::result;

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

/// A builder for a shell command.
///
/// Possibilities include:
///
/// - an executable on the path,
/// - a user-defined alias,
/// - a shell builtin, or
/// - a shell function.
///
/// All of these will be wrapped up in `Func`.
///
pub struct CommandBuilder<'e, Iter, Args>
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>,
{
    env: Option<&'e mut Environment>,
    args: Option<Args>,
    f: Box<dyn FnMut(&mut Environment, Args) -> Result + 'e>,
}

impl <'e, Iter, Args> CommandBuilder<'e, Iter, Args>
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>,
{
    /// Construct a `CommandBuilder` for the given function.
    ///
    pub fn new<Func>(f: Box<Func>) -> CommandBuilder<'e, Iter, Args>
        where Func: FnMut(&mut Environment, Args) -> Result + 'e,
    {
        CommandBuilder {
            env: None,
            args: None,
            f: f,
        }
    }

    /// Supply the given arguments as those for this command.
    ///
    pub fn args(&mut self, args: Args) -> &mut Self {
        self.args = Some(args);
        self
    }

    /// Use the given `Environment` for this command.
    ///
    fn env(&mut self, env: &'e mut Environment) -> &mut Self {
        self.env = Some(env);
        self
    }

    /// Execute this command.
    ///
    fn execute(mut self) -> Result {
        match (self.env, self.args) {
            (Some(env), Some(args)) => (self.f)(env, args),
            _ => Err(Error::Unknown),
        }
    }
}
