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
    Error,
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn exec<Iter, Args>(_env: &mut Environment, args: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
{
    let mut args_iter = args.into_iter();
    match args_iter.next() {
        Some(_command) => {
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
        },
        None =>  {
            // TODO e.g., exec 2>&1 should make all stderr go to stdout in the shell
            Ok(ExitStatus::Success(0))
        },
    }
}
