//! Support for executable units from the pat.
//!
use super::{
    Context,
    Error,
    ExitStatus,
    Result,
};

use std::{
    ffi::OsStr,
    process,
};

/// An executable on the path.
///
pub struct Executable {
    command: process::Command,
}

impl Executable {
    /// Constructs a new command for the executable at the given path.
    ///
    pub fn new<P: AsRef<OsStr>>(command_path: P) -> Executable {
        Executable {
            command: process::Command::new(command_path.as_ref()),
        }
    }

    /// Execute this command within the given context.
    ///
    pub fn execute(mut self, Context { env, args }: Context) -> Result {
        self.command
            .envs(env.exported_vars())
            .current_dir(env.working_directory())
            .args(args)
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
