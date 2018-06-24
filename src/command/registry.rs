//! A registry for commands.
//!
use std::{
    fs,
    os::unix::{
        fs::PermissionsExt,
    },
    path::PathBuf,
};

use super::{
    builtin::*,

    Command,
    Error,
    Executable,
    ExitStatus,
};

use environment::{
    Environment,
};

use strings::ShellString;

/// A registry of commands.
///
/// A registry maintains the builtins, user-defined aliases, and so on. More generally, it's
/// the entrypoint for executing a command on the shell.
///
pub struct Registry {
    executable_paths: Vec<PathBuf>,
    working_directory: PathBuf,
}

impl Registry {
    /// Constructs a new registry.
    ///
    pub fn for_env(env: &Environment) -> Registry {
        Registry {
            executable_paths: env.paths().clone(),
            working_directory: env.working_directory().clone(),
        }
    }

    /// Executes `command` within this environment.
    ///
    /// If found, returns the exit status of the command.
    ///
    pub fn execute(&self, env: &mut Environment, command: &String, args: Vec<ShellString>) -> Result<ExitStatus, Error> {
        match command.as_ref() {
            "cd" => {
                let args = ShellString::to_string_vec(args.into_iter(), env);
                match args {
                    Some(args) => Cd::new().args(args).env(env).execute(),
                    None => Err(Error::Unknown),
                }
            },

            "exec" => {
                let args = ShellString::to_string_vec(args.into_iter(), &env);
                match args {
                    Some(args) => Exec::new().args(args).env(env).execute(),
                    None => Err(Error::Unknown),
                }
            },

            "exit" => {
                let args = ShellString::to_string_vec(args.into_iter(), &env);
                match args {
                    Some(args) => Exit::new().args(args).env(env).execute(),
                    None => Err(Error::Unknown),
                }
            },

            _ => {
                let absolute_command = self.find_executable(&PathBuf::from(command));
                if let Some(path) = absolute_command {
                    match ShellString::to_string_vec(args.into_iter(), &env) {
                        Some(interpolated_args) => {
                            Executable::new(path)
                                .args(interpolated_args)
                                .env(env)
                                .execute()
                        },
                        None => Err(Error::Unknown),
                    }
                } else {
                    Err(Error::UnknownCommand)
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
            let mut executables_in_path = self.executable_paths.iter().map(|path| self.executable(path.join(command)));
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
    #[cfg(target_famlily = "unix")]
    #[test]
    fn test_execute_finds_and_executes_relative_command() {
        let env = Environment::from_existing_env();
        let registry = Registry::for_env(&env);
        assert_eq!(Some(0), registry.execute(&env, "true", vec![]).unwrap().code());
    }

    #[cfg(target_famlily = "unix")]
    #[test]
    fn test_execute_returns_error_when_not_on_path() {
        let env = Environment::new(HashMap::new());
        let registry = Registry::for_env(&env);
        assert_eq!(Err(Error::CommandNotFound), env.execute(&env, "true", vec![]));
    }

    #[cfg(target_famlily = "unix")]
    #[test]
    fn test_execute_executes_absolute_command() {
        let env = Environment::new(HashMap::new());
        let registry = Registry::for_env(&env);
        assert_eq!(Some(1), env.execute(&env, "/usr/bin/false", vec![]).unwrap().code());
    }
}
