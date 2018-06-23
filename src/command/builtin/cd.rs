use std::{
    env,
    path::PathBuf,
};

builtin!(
    Cd,
    self {
        match self.env {
            Some(ref mut env) => {
                let new_dir = match self.args.get(0) {
                    Some(dir) => PathBuf::from(dir).canonicalize().ok(),
                    None => env::home_dir(),
                };

                match new_dir {
                    Some(dir) => {
                        env.set_working_directory(dir);
                        Ok(ExitStatus::Success(0))
                    },
                    None => Ok(ExitStatus::Success(0))
                }
            },
            None => Err(Error::Unknown)
        }
    }
);
