//! Encapsulates the environment in which commands within a shell executes.
//!
use std::{
    borrow::Borrow,
    collections::HashMap,
    env,
    fs,
    os::unix::{
        fs::PermissionsExt,
        process::ExitStatusExt,
    },
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
#[derive(Debug, PartialEq)]
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
    working_directory: PathBuf,
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

            // TODO something better than '/'?
            working_directory: env::current_dir().unwrap_or(PathBuf::from("/")),
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

    /// Returns the current working directory.
    ///
    pub fn working_directory(&self) -> &PathBuf {
        &self.working_directory
    }

    /// Gets the value of a variable from this environment.
    ///
    pub fn var<S: Borrow<String>>(&self, name: &S) -> Option<String> {
        match self.vars.get(name.borrow()) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    /// Executes `command` within this environment.
    ///
    /// If found, returns the exit status of the command.
    ///
    pub fn execute<S>(&mut self, command: &String, args: S) -> Result<ExitStatus, CommandError>
        where S: IntoIterator<Item = ShellString>,
    {
        // TODO refactor the builtins out of here into somewhere nicer
        match command.as_ref() {
            "cd" => {
                let new_dir = match args.into_iter().take(1).next() {
                    Some(dir) => 
                        dir.to_string(&self)
                           .and_then(|v| PathBuf::from(v).canonicalize().ok()),
                    None => env::home_dir(),
                };

                match new_dir {
                    Some(dir) => {
                        self.working_directory = dir;
                        Ok(ExitStatus::from_raw(0))
                    },
                    None => Ok(ExitStatus::from_raw(1))
                }
            },
            _ => {
                let absolute_command = self.find_executable(&PathBuf::from(command));
                if let Some(path) = absolute_command {
                    let mapped_args = args.into_iter().map(|a| a.to_string(&self));
                    let interpolated_args = mapped_args.collect::<Option<Vec<_>>>().unwrap_or(vec![]);
                    Command::new(path)
                        .args(interpolated_args)
                        .envs(self.vars.clone().iter())
                        .current_dir(&self.working_directory)
                        .status()
                        .map_err(|_| CommandError::Unknown)
                } else {
                    Err(CommandError::CommandNotFound)
                }
            }
        }
    }

    /// Finds an executable within this environment.
    ///
    fn find_executable(&self, command: &PathBuf) -> Option<PathBuf> {
        // TODO make this nicer
        if command.is_absolute() {
            if self.is_executable(&command) {
                Some(command.clone())
            } else {
                None
            }
        } else if command.parent() != Some(&PathBuf::from("")) {
            // TODO maybe I need my own path representation to better separate absolute / relative
            // / PATH-based paths (can be done at the parser level)
            let command_in_working_directory = self.working_directory.join(command);
            if self.is_executable(&command_in_working_directory) {
                Some(command_in_working_directory)
            } else {
                None
            }
        } else {
            match self.paths.iter().find(|path| self.is_executable(&path.join(command))) {
                Some(path) => Some(path.join(command)),
                None => {
                     let command_in_working_directory = self.working_directory.join(command);
                     if self.is_executable(&command_in_working_directory) {
                         Some(command_in_working_directory)
                     } else {
                         None
                     }
                },
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uses_vars_from_given_map() {
        let expected_paths = vec!["/bin", "/usr/bin", "/usr/local/bin"];
        let joined_paths = env::join_paths(&expected_paths).unwrap().into_string().unwrap();

        let mut vars = HashMap::new();
        vars.insert("PATH".to_owned(), joined_paths.clone());
        vars.insert("HOME".to_owned(), "/Users/foo".to_owned());

        let env = Environment::new(vars);
        assert_eq!(expected_paths, env.paths.iter().map(|v| v.as_os_str()).collect::<Vec<_>>());
        assert_eq!(Some(joined_paths), env.var(&"PATH".to_owned()));
        assert_eq!(Some("/Users/foo".to_owned()), env.var(&"HOME".to_owned()));
        assert_eq!(None, env.var(&"TMPDIR".to_owned()));
    }

    #[cfg(target_famlily = "unix")]
    #[test]
    fn test_execute_finds_and_executes_relative_command() {
        let env = Environment::from_existing_env();
        assert_eq!(Some(0), env.execute("true", vec![]).unwrap().code());
    }

    #[cfg(target_famlily = "unix")]
    #[test]
    fn test_execute_returns_error_when_not_on_path() {
        let env = Environment::new(HashMap::new());
        assert_eq!(Err(CommandError::CommandNotFound), env.execute("true", vec![]));
    }

    #[cfg(target_famlily = "unix")]
    #[test]
    fn test_execute_executes_absolute_command() {
        let env = Environment::new(HashMap::new());
        assert_eq!(Some(1), env.execute("/usr/bin/false", vec![]).unwrap().code());
    }
}
