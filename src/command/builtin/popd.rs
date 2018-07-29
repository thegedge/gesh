use command::{
    Context,
    ExitStatus,
    Result,
};

use super::cd;

pub fn popd(Context { env, args }: Context) -> Result {
    match args.len() {
        0 => {
            match env.pop_directory() {
                Some(dir) => {
                    match dir.into_os_string().into_string() {
                        Ok(dir_string) => cd(Context { env, args: vec![dir_string] }),
                        _ => Ok(ExitStatus::Success(3)),
                    }
                },
                None => Ok(ExitStatus::Success(2)),
            }
        },
        _ => Ok(ExitStatus::Success(1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use environment::Environment;

    #[test]
    fn test_popd_returns_error_with_wrong_number_of_arguments() {
        let env = &mut Environment::empty();
        env.push_directory(PathBuf::from("src"));

        let args = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];

        let result = popd(Context { env, args });

        assert_eq!(Ok(ExitStatus::Success(1)), result);
    }

    #[test]
    fn test_popd_returns_error_if_nothing_on_stack() {
        let env = &mut Environment::empty();
        let args = vec![];

        let result = popd(Context { env, args });

        assert_eq!(Ok(ExitStatus::Success(2)), result);
    }

    #[test]
    fn test_popd_changes_working_directory_to_top_of_stack() {
        let env = &mut Environment::empty();
        env.push_directory(PathBuf::from("src"));

        let args = vec![];

        let result = popd(Context { env, args });

        assert_eq!(Ok(ExitStatus::Success(0)), result);
        assert_eq!(PathBuf::from("src").canonicalize().unwrap(), *env.working_directory());
    }
}
