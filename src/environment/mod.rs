//! Encapsulates the environment in which commands within a shell executes.
//!
use std::{
    borrow::Borrow,
    collections::HashMap,
    env,
    fs,
    os::unix::{
        fs::PermissionsExt,
    },
    path::PathBuf,
};

use super::{
    command::*,
    strings::ShellString,
};

pub use command::{
    Command,
    Error as CommandError,
    ExitStatus,
};

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

    /// Sets the current working directory.
    ///
    pub fn set_working_directory(&mut self, path: PathBuf) {
        self.working_directory = path;
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

    /// Gets the value of a variable from this environment.
    ///
    pub fn vars(&self) -> HashMap<String, String> {
        self.vars.clone()
    }

    /// Executes `command` within this environment.
    ///
    /// If found, returns the exit status of the command.
    ///
    pub fn execute(&mut self, command: &String, args: Vec<ShellString>) -> Result<ExitStatus, CommandError> {
        // TODO refactor the builtins out of here into somewhere nicer
        match command.as_ref() {
            "cd" => {
                let args = ShellString::to_string_vec(args.into_iter(), &self);
                match args {
                    Some(args) => Cd::new().args(args).env(self).execute(),
                    None => Err(CommandError::Unknown),
                }
            },

            "exec" => {
                let args = ShellString::to_string_vec(args.into_iter(), &self);
                match args {
                    Some(args) => Exec::new().args(args).env(self).execute(),
                    None => Err(CommandError::Unknown),
                }
            },

            "exit" => {
                let args = ShellString::to_string_vec(args.into_iter(), &self);
                match args {
                    Some(args) => Exit::new().args(args).env(self).execute(),
                    None => Err(CommandError::Unknown),
                }
            },

            _ => {
                let absolute_command = self.find_executable(&PathBuf::from(command));
                if let Some(path) = absolute_command {
                    match ShellString::to_string_vec(args.into_iter(), &self) {
                        Some(interpolated_args) => {
                            Executable::new(path)
                                .args(interpolated_args)
                                .env(self)
                                .execute()
                        },
                        None => Err(CommandError::Unknown),
                    }
                } else {
                    Err(CommandError::UnknownCommand)
                }
            }
        }
    }

    /// Finds an executable on the path.
    ///
    pub fn find_executable(&self, command: &PathBuf) -> Option<PathBuf> {
        // TODO make this nicer
        if command.is_absolute() {
            self.executable(command.clone())
        } else if command.parent() != Some(&PathBuf::from("")) {
            let command_in_working_directory = self.working_directory.join(command);
            self.executable(command_in_working_directory)
        } else {
            let mut executables_in_path = self.paths.iter().map(|path| self.executable(path.join(command)));
            match executables_in_path.find(|path| path.is_some()) {
                Some(executable) => executable,
                None => {
                    let command_in_working_directory = self.working_directory.join(command);
                    self.executable(command_in_working_directory)
                }
            }
        }
    }

    /// Returns whether or not the file is executable.
    ///
    fn executable(&self, command: PathBuf) -> Option<PathBuf> {
        // TODO make this work for more than Unix
        // TODO perhaps need to check if current user can execute the command
        match fs::metadata(&command) {
            Ok(ref metadata) if (metadata.permissions().mode() & 0o111) != 0 => {
                Some(command)
            },
            _ => None,
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
