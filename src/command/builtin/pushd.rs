use std::{
    path::PathBuf,
};

use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn pushd(Context { env, args }: Context) -> Result {
    match args.into_iter().next() {
        Some(dir) => {
            env.push_directory(PathBuf::from(dir));
            Ok(ExitStatus::Success(0))
        },
        None => Ok(ExitStatus::Success(1)),
    }
}
