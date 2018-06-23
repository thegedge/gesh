use std::{
    path::PathBuf,
    process,
    os::unix::{
        process::CommandExt,
    },
};

use environment::CommandError;

builtin!(
    Exec,
    self {
        if self.args.len() == 0 {
            // TODO e.g., exec 2>&1 should make all stderr go to stdout in the shell
            Ok(ExitStatus::Success(0))
        } else {
            match &self.env {
                Some(env) => {
                    let absolute_command = env.find_executable(&PathBuf::from(&self.args[0]));
                    if let Some(path) = absolute_command {
                        process::Command::new(path)
                            .args(self.args.iter().skip(1))
                            .envs(env.vars())
                            .current_dir(env.working_directory())
                            .exec();

                        Err(CommandError::Unknown)
                    } else {
                        Err(CommandError::UnknownCommand)
                    }
                },
                None => Err(CommandError::Unknown),
            }
        }
    }
);
