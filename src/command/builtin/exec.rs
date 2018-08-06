use std::{
    path::PathBuf,
    process,
    os::unix::{
        process::CommandExt,
    },
};

use command::{
    Context,
    Error,
    ExitStatus,
    Result,
};

pub fn exec(Context { args, env, registry, .. }: Context) -> Result {
    if args.is_empty() {
        // TODO e.g., exec 2>&1 should make all stderr go to stdout in the shell
        Ok(ExitStatus::Success(0))
    } else {
        let absolute_command = registry.find_executable(&PathBuf::from(&args[0]));
        if let Some(path) = absolute_command {
            process::Command::new(path)
                .args(args.iter().skip(1))
                .envs(env.exported_vars())
                .current_dir(env.working_directory())
                .exec();

            Err(Error::Unknown)
        } else {
            Err(Error::UnknownCommand)
        }
    }
}
