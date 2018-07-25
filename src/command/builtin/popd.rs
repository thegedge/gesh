use command::{
    ExitStatus,
    Result,
};

use environment::Environment;
use super::cd;

pub fn popd<Args>(env: &mut Environment, _: Args) -> Result
    where Args: IntoIterator<Item = String>
{
    match env.pop_directory() {
        Some(dir) => {
            match dir.into_os_string().into_string() {
                Ok(dir_string) => cd(env, vec![dir_string]),
                _ => Ok(ExitStatus::Success(1)),
            }
        },
        None => Ok(ExitStatus::Success(1)),
    }
}
