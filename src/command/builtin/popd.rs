use command::{
    Context,
    ExitStatus,
    Result,
};

use super::cd;

pub fn popd(Context { env, .. }: Context) -> Result {
    match env.pop_directory() {
        Some(dir) => {
            match dir.into_os_string().into_string() {
                Ok(dir_string) => cd(Context { env, args: vec![dir_string] }),
                _ => Ok(ExitStatus::Success(1)),
            }
        },
        None => Ok(ExitStatus::Success(1)),
    }
}
