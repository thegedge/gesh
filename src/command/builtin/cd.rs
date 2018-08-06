use std::{
    env,
    path::PathBuf,
};

use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn cd(Context { env, args, .. }: Context) -> Result {
    let new_dir = match args.len() {
        0 => env::home_dir(),
        1 => PathBuf::from(&args[0]).canonicalize().ok(),
        _ => return Ok(ExitStatus::Success(2)),
    };

    match new_dir {
        Some(dir) => {
            env.set_working_directory(dir);
            Ok(ExitStatus::Success(0))
        },
        None => Ok(ExitStatus::Success(1))
    }
}

#[cfg(test)]
mod tests {
    use command::Registry;
    use environment::Environment;
    use std::env;
    use super::*;

    #[test]
    fn test_cd_switches_to_home_dir_with_no_arguments() {
        let env = &mut Environment::from_existing_env();
        let args = vec![];
        let registry = &Registry::for_env(&env);

        let result = cd(Context { env, args, registry });

        assert_eq!(Ok(ExitStatus::Success(0)), result);
        assert_eq!(env::home_dir().unwrap(), *env.working_directory());
    }

    #[test]
    fn test_cd_switches_to_canonical_form_of_given_directory() {
        let env = &mut Environment::from_existing_env();
        let args = vec![String::from(env::temp_dir().to_string_lossy())];
        let registry = &Registry::for_env(&env);

        let result = cd(Context { env, args, registry });

        assert_eq!(Ok(ExitStatus::Success(0)), result);
        assert_eq!(env::temp_dir().canonicalize().unwrap(), *env.working_directory());
    }

    #[test]
    fn test_cd_returns_error_when_too_many_arguments() {
        let env = &mut Environment::from_existing_env();
        let args = vec!["too".to_owned(), "many".to_owned(), "arguments".to_owned()];
        let original_working_directory = env.working_directory().clone();
        let registry = &Registry::for_env(&env);

        let result = cd(Context { env, args, registry });

        assert_eq!(Ok(ExitStatus::Success(2)), result);
        assert_eq!(original_working_directory, *env.working_directory());
    }

    #[test]
    fn test_cd_returns_error_when_path_doesnt_exist() {
        let env = &mut Environment::from_existing_env();
        let args = vec!["not/a/directory/that/exists".to_owned()];
        let original_working_directory = env.working_directory().clone();
        let registry = &Registry::for_env(&env);

        let result = cd(Context { env, args, registry });

        assert_eq!(Ok(ExitStatus::Success(1)), result);
        assert_eq!(original_working_directory, *env.working_directory());
    }
}
