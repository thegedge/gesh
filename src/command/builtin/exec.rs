/*
use std::{
    path::PathBuf,
    process,
    os::unix::{
        process::CommandExt,
    },
};
*/

use command::{
    Context,
    Error,
    ExitStatus,
    Result,
};

pub fn exec(Context { args, .. }: Context) -> Result {
    if args.is_empty() {
        // TODO e.g., exec 2>&1 should make all stderr go to stdout in the shell
        Ok(ExitStatus::Success(0))
    } else {
        Err(Error::Unknown)
        // TODO find way to pass registry
        /*
        match &self.env {
            Some(env) => {
                let absolute_command = env.find_executable(&PathBuf::from(&self.args[0]));
                if let Some(path) = absolute_command {
                    process::Command::new(path)
                        .args(self.args.iter().skip(1))
                        .envs(env.vars())
                        .current_dir(env.working_directory())
                        .exec();

                    Err(Error::Unknown)
                } else {
                    Err(Error::UnknownCommand)
                }
            },
            None => Err(Error::Unknown),
        }
        */
    }
}
