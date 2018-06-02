//! All things dealing with readline.
//!
//! Readline provides traits and implementations to support readline a line of text
//! for the shell. Important features of an implementation:
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
    Io(io::Error),

    /// EOF marker was input
    Eof(),

    /// Received a SIGINT
    Interrupted(),

    /// Some other, unknown or uncategorized error
    Other(),
}

/// Result used by most `Reader` methods
type Result<T> = result::Result<T, Error>;

/// Abstraction for readline input
pub trait Reader {
    /// Gets the next command from the input
    ///
    /// # Errors
    /// Everything in `Error`
    ///
    fn get(&mut self) -> Result<String>;
}
