use std::{
    env,
    path::PathBuf,
};

use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn cd(Context { env, args }: Context) -> Result {
    let new_dir = match args.len() {
        0 => env::home_dir(),
        1 => PathBuf::from(&args[0]).canonicalize().ok(),
        _ => return Ok(ExitStatus::Success(1)),
    };

    match new_dir {
        Some(dir) => {
            env.set_working_directory(dir);
            Ok(ExitStatus::Success(0))
        },
        None => Ok(ExitStatus::Success(0))
    }
}
