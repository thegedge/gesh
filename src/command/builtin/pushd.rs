use std::{
    path::PathBuf,
};

use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn pushd<Args>(env: &mut Environment, args: Args) -> Result
    where Args: IntoIterator<Item = String>
{
    match args.into_iter().next() {
        Some(dir) => {
            env.push_directory(PathBuf::from(dir));
            Ok(ExitStatus::Success(0))
        },
        None => Ok(ExitStatus::Success(1)),
    }
}
