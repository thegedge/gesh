use command::{
    ExitStatus,
    Result,
};

use environment::Environment;
use super::cd;

pub fn popd<Iter, Args>(env: &mut Environment, _: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
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
