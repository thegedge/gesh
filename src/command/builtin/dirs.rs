use command::{
    Error,
    ExitStatus,
    Result,
};

use environment::Environment;

pub fn dirs<Iter, Args>(env: &mut Environment, _args: Args) -> Result
    where
        Iter: Iterator<Item = String>,
        Args: IntoIterator<Item = String, IntoIter = Iter>
{
    let stack = env.directory_stack();
    if stack.len() > 0 {
        let mut iter = stack.iter();
        match iter.next().unwrap().to_str() {
            Some(string) => print!("{}", string),
            None => return Err(Error::Unknown),
        };

        while let Some(dir) = iter.next() {
            match dir.to_str() {
                Some(string) => print!(" {}", string),
                None => return Err(Error::Unknown),
            };
        }
    }
    print!("\n");
    Ok(ExitStatus::Success(0))
}
