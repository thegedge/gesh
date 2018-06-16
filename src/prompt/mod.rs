//! All things dealing with the prompt.
//!
//! This module provides traits and implementations to support rendering the prompt and reading
//! commands from the user. Important features of an implementation:
//!
//! * Command history
//! * Completion,
//! * Modes (vi/emacs)
//!
use std::io;
use std::result;

pub mod rustyline;

/// Errors when reading commands
///
#[derive(Debug)]
pub enum Error {
    /// Generic I/O error when trying to get input from the TTY
    ///
    Io(io::Error),

    /// EOF marker was input
    ///
    Eof(),

    /// Received a SIGINT
    ///
    Interrupted(),

    /// Some other, unknown or uncategorized error
    ///
    Other(),
}

/// Result used by most `Prompt` methods
///
type Result<T> = result::Result<T, Error>;

/// Abstraction for an input prompt
pub trait Prompt {
    /// Sets the text displayed for the input prompt.
    ///
    fn set_prompt(&mut self, String);

    /// Gets the next command from the input
    ///
    /// # Errors
    /// Everything in `Error`
    ///
    fn get(&mut self) -> Result<String>;
}
