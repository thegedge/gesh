use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn exit(Context { args, .. }: Context) -> Result {
    let status = match args.len() {
        0 => 0,
        1 => args[0].parse().unwrap_or(255),
        _ => return Ok(ExitStatus::Success(1)),
    };
    Ok(ExitStatus::ExitWith(status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use environment::Environment;

    #[test]
    fn test_exit_returns_exit_with_given_status() {
        let env = &mut Environment::from_existing_env();
        let args = vec!["77".to_owned()];

        let result = exit(Context { env, args });

        assert_eq!(Ok(ExitStatus::ExitWith(77)), result);
    }

    #[test]
    fn test_exit_returns_zero_exit_status_with_no_arguments() {
        let env = &mut Environment::from_existing_env();
        let args = vec![];

        let result = exit(Context { env, args });

        assert_eq!(Ok(ExitStatus::ExitWith(0)), result);
    }

    #[test]
    fn test_exit_returns_nonzero_exit_status_with_non_integral_argument() {
        let env = &mut Environment::from_existing_env();
        let args = vec!["abc".to_owned()];

        let result = exit(Context { env, args });

        assert_eq!(Ok(ExitStatus::ExitWith(255)), result);
    }

    #[test]
    fn test_exit_returns_error_with_too_many_arguments() {
        let env = &mut Environment::from_existing_env();
        let args = vec!["a".to_owned(), "b".to_owned()];

        let result = exit(Context { env, args });

        assert_eq!(Ok(ExitStatus::Success(1)), result);
    }
}
