//! Support for executable units from the pat.
//!
use super::{
    Error,
    ExecutableUnit,
    ExitStatus,
};

use environment::{
    Environment
};

use std::{
    ffi::OsStr,
    process,
};

/// An executable unit from the path
///
pub struct Command<'e> {
    env: Option<&'e Environment>,
    command: process::Command,
}

impl <'e> Command<'e> {
    /// Constructs a new path command for the executable at the given path.
    ///
    pub fn new<P: AsRef<OsStr>>(command_path: P) -> Command<'e> {
        Command {
            env: None,
            command: process::Command::new(command_path.as_ref()),
        }
    }
}

impl <'e> ExecutableUnit<'e> for Command<'e> {
    fn args(&mut self, args: Vec<String>) -> &mut Self {
        self.command.args(args);
        self
    }

    fn env<'v: 'e>(&mut self, env: &'v Environment) -> &mut Self {
        self.env = Some(env);
        self.command.envs(env.vars());
        self.command.current_dir(env.working_directory());
        self
    }

    fn execute(&mut self) -> Result<ExitStatus, Error> {
        self.command
            .status()
            .map(|status| ExitStatus::Success(status.code().unwrap_or(1) as u32))
            .map_err(|_| Error::Unknown)
    }
}

impl From<process::ExitStatus> for ExitStatus {
    fn from(status: process::ExitStatus) -> Self {
        ExitStatus::Success(status.code().unwrap_or(1) as u32)
    }
}
