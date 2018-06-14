//! Encapsulates the environment in which commands within a shell executes.
//!
use std::{
    collections::HashMap,
    env,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{
        Command,
        ExitStatus,
    },
};

use super::{
    strings::ShellString,
};

/// The error type for environment errors.
///
#[derive(Debug)]
pub enum CommandError {
    /// Command wasn't found on the path
    CommandNotFound,

    /// Generic error for unknown/uncateogrized errors
    Unknown,
}

/// Supports executing commands within the context of a specific environment.
///
pub struct Environment {
    paths: Vec<PathBuf>,
    vars: HashMap<String, String>,
}

impl Environment {
    /// Creates a new environment.
    ///
    /// Defaults to containing the same paths as the shell's PATH environment variable.
    ///
    pub fn new(vars: HashMap<String, String>) -> Environment {
        let paths = match vars.get("PATH") {
            Some(paths) => env::split_paths(&paths).collect(),
            None => vec![]
        };

        Environment {
            paths: paths,
            vars: vars,
        }
    }

    ///
    ///
    pub fn from_existing_env() -> Environment {
        let vars = env::vars();
        let capacity = vars.size_hint().1.unwrap_or(0);
        Self::new(vars.fold(
            HashMap::with_capacity(capacity),
            |mut result, (name, value)| {
                result.insert(name, value);
                result
            }
        ))
    }

    /// Gets the value of a variable from this environment.
    ///
    pub fn var(&self, name: &String) -> Option<String> {
        match self.vars.get(name) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    /// Executes `command` within this environment.
    ///
    /// If found, returns the exit status of the command. 
    ///
    pub fn execute<T, S>(&self, command: T, args: S) -> Result<ExitStatus, CommandError>
        where T: Into<PathBuf>,
              S: IntoIterator<Item = ShellString>,
    {
        let absolute_command = self.find_executable(&command.into());
        if let Some(path) = absolute_command {
            let mapped_args = args.into_iter().map(|a| a.to_string(&self));
            let interpolated_args = mapped_args.collect::<Option<Vec<_>>>().unwrap_or(vec![]);
            Command::new(path)
                .args(interpolated_args)
                .envs(self.vars.clone().iter())
                .status()
                .map_err(|_| CommandError::Unknown)
        } else {
            Err(CommandError::CommandNotFound)
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
