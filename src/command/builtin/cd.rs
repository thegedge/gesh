use std::{
    env,
    path::PathBuf,
};

use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn cd<Iter, Args>(env: &mut Environment, args: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
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
