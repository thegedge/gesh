//! Methods and structures to work with a command's context, which includes
//! its environment, file redirections, arguments, and so on.
//!
use command::Registry;
use environment::Environment;

/// A context for running commands.
///
pub struct Context<'c> {
    pub env: &'c mut Environment,
    pub args: Vec<String>,
    pub registry: &'c Registry,
}
