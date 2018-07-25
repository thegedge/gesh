use std::{
    env,
    path::PathBuf,
};

use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn cd<Args>(env: &mut Environment, args: Args) -> Result
    where Args: IntoIterator<Item = String>
{
    let new_dir = match args.into_iter().next() {
        Some(dir) => PathBuf::from(dir).canonicalize().ok(),
        None => env::home_dir(),
    };

    match new_dir {
        Some(dir) => {
            env.set_working_directory(dir);
            Ok(ExitStatus::Success(0))
        },
        None => Ok(ExitStatus::Success(0))
    }
}
