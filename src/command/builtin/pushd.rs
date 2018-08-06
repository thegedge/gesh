use std::{
    path::PathBuf,
};

use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn pushd(Context { env, args, .. }: Context) -> Result {
    match args.len() {
        0 => {
            // TODO swap top two paths
            Ok(ExitStatus::Success(1))
        },
        1 => {
            // TODO check for existence of path before pushing
            env.push_directory(PathBuf::from(&args[0]));
            Ok(ExitStatus::Success(0))
        },
        _ => Ok(ExitStatus::Success(1)),
    }
}

#[cfg(test)]
mod tests {
    use command::Registry;
    use environment::Environment;
    use super::*;

    #[test]
    fn test_pushd_adds_given_directory_to_stack() {
        let env = &mut Environment::empty();
        let args = vec!["./src".to_owned()];
        let registry = &Registry::for_env(&env);

        let result = pushd(Context { env, args, registry });

        assert_eq!(Ok(ExitStatus::Success(0)), result);
        assert_eq!(Some(&PathBuf::from("./src")), env.directory_stack().last());
    }

    #[test]
    fn test_pushd_returns_error_with_too_many_arguments() {
        let env = &mut Environment::empty();
        let args = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        let registry = &Registry::for_env(&env);

        let result = pushd(Context { env, args, registry });

        assert_eq!(Ok(ExitStatus::Success(1)), result);
    }
}
