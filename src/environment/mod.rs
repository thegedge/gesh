//! Encapsulates the environment in which commands within a shell executes.
//!
use std::{
    env,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{
        Command,
        ExitStatus,
    },
    result,
};

use super::{
    strings::ShellString,
};

/// The error type for environment errors.
///
#[derive(Debug)]
pub enum Error {
    /// Command wasn't found on the path
    CommandNotFound,

    /// Generic error for unknown/uncateogrized errors
    Unknown,
}

/// A specialized result type for environment functions.
///
pub type Result<T> = result::Result<T, Error>;

/// Supports executing commands within the context of a specific environment.
///
pub struct Environment {
    paths: Vec<PathBuf>
}

impl Environment {
    /// Creates a new environment.
    ///
    /// Defaults to containing the same paths as the shell's PATH environment variable.
    ///
    pub fn new() -> Environment {
        let paths = match env::var_os("PATH") {
            Some(paths) => env::split_paths(&paths).collect(),
            None => vec![]
        };

        Environment {
            paths: paths
        }
    }

    /// Executes `command` within this environment.
    ///
    /// If found, returns the exit status of the command. 
    ///
    pub fn execute<T, S>(&self, command: T, args: S) -> Result<ExitStatus>
        where T: Into<PathBuf>,
              S: IntoIterator<Item = ShellString>,
    {
        let absolute_command = self.find_executable(&command.into());
        if let Some(path) = absolute_command {
            let interpolated_args = args.into_iter().map(|a| a.to_string(&self));
            Command::new(path).args(interpolated_args).status().map_err(|_| Error::Unknown)
        } else {
            Err(Error::CommandNotFound)
        }
    }

    /// Finds an executable within this environment.
    ///
    fn find_executable(&self, command: &PathBuf) -> Option<PathBuf> {
        if command.is_absolute() {
            Some(command.clone())
        } else {
            self.paths.iter().find(
                |path| self.is_executable(&path.join(command))
            ).map(
                |path| path.join(command)
            )
        }
    }

    /// Returns whether or not the file is executable.
    ///
    fn is_executable(&self, command: &PathBuf) -> bool {
        // TODO make this work for more than Unix
        match fs::metadata(command) {
            Ok(metadata) => {
                (metadata.permissions().mode() & 0o111) != 0
            },
            _ => false,
        }
    }
}
