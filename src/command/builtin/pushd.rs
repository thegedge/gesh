use std::{
    path::PathBuf,
};

use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn pushd<Iter, Args>(env: &mut Environment, args: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
{
    match args.into_iter().next() {
        Some(dir) => {
            env.push_directory(PathBuf::from(dir));
            Ok(ExitStatus::Success(0))
        },
        None => Ok(ExitStatus::Success(1)),
    }
}
