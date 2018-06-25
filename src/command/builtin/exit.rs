use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn exit<Iter, Args>(_env: &mut Environment, args: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
{
    let status = match args.into_iter().next() {
        Some(status) => status.parse().unwrap_or(255),
        None => 0,
    };
    Ok(ExitStatus::ExitWith(status))
}
