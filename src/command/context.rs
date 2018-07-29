//! Methods and structures to work with a command's context, which includes
//! its environment, file redirections, arguments, and so on.
//!
use environment::Environment;

/// A context for running commands.
///
pub struct Context<'e> {
    pub env: &'e mut Environment,
    pub args: Vec<String>,
}
