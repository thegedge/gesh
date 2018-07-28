use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn export(Context { env, args }: Context) -> Result {
    if args.is_empty() {
        for (key, value) in env.exported_vars() {
            println!("{}={}", key, value);
        }
    } else {
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
    }

    Ok(ExitStatus::Success(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use environment::Environment;

    #[test]
    fn test_export_prints_all_exported_vars_with_no_arguments() {
        let env = &mut Environment::empty();
        let args = vec![];

        let result = export(Context { env, args });

        assert_eq!(Ok(ExitStatus::Success(0)), result);
    }

    #[test]
    fn test_export_exports_all_vars_given_as_arguments() {
        let env = &mut Environment::empty();
        env.set("FOO".to_owned(), "bar".to_owned());
        env.set("SPAM".to_owned(), "10 eggs".to_owned());
        env.set("DONT_EXPORT_ME".to_owned(), "plz".to_owned());

        let args = vec![
            "FOO".to_owned(),
            "SPAM=11 eggs".to_owned(),
        ];

        let result = export(Context { env, args });

        assert_eq!(Ok(ExitStatus::Success(0)), result);
        assert_eq!(Some(&"bar".to_owned()), env.exported_vars().get("FOO"));
        assert_eq!(Some(&"11 eggs".to_owned()), env.exported_vars().get("SPAM"));
        assert_eq!(None, env.exported_vars().get("DONT_EXPORT_ME"));
    }
}
