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
    for arg in args {
        let mut split = arg.split('=');
        let name_part = split.next();
        let value_part = split.next();
        match (name_part, value_part) {
            (Some(name), Some(value)) => {
                env.set(name.to_owned(), value.to_owned());
                env.export(name.to_owned());
            },
            (Some(name), None) => {
                env.export(name.to_owned());
            },
            _ => unreachable!(),
        };
    }

    Ok(ExitStatus::Success(0))
}
