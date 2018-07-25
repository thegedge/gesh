use command::{
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn export<Args>(env: &mut Environment, args: Args) -> Result
    where Args: IntoIterator<Item = String>
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
