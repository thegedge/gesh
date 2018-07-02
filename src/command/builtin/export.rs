use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn export<Iter, Args>(env: &mut Environment, args: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
{
    for name in args {
        env.export(name);
    }
    Ok(ExitStatus::Success(0))
}
