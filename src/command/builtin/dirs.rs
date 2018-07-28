use command::{
    Context,
    Error,
    ExitStatus,
    Result,
};

pub fn dirs(Context { env, .. }: Context) -> Result {
    let stack = env.directory_stack();
    if !stack.is_empty() {
        let mut iter = stack.iter();
        match iter.next().unwrap().to_str() {
            Some(string) => print!("{}", string),
            None => return Err(Error::Unknown),
        };

        for dir in iter {
            match dir.to_str() {
                Some(string) => print!(" {}", string),
                None => return Err(Error::Unknown),
            };
        }
    }
    println!();

    Ok(ExitStatus::Success(0))
}
