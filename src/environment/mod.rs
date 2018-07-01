//! Encapsulates the environment in which commands within a shell executes.
//!
use std::{
    borrow::Borrow,
    collections::HashMap,
    env,
    path::PathBuf,
};

/// Supports executing commands within the context of a specific environment.
///
#[derive(Clone)]
pub struct Environment {
    paths: Vec<PathBuf>,
    vars: HashMap<String, String>,
    working_directory: PathBuf,
    directory_stack: Vec<PathBuf>,
}

impl Environment {
    /// Creates a new environment.
    ///
    /// Defaults to containing the same paths as the shell's PATH environment variable.
    ///
    pub fn new(vars: HashMap<String, String>) -> Environment {
        let paths = match vars.get("PATH") {
            Some(paths) => env::split_paths(&paths).collect(),
            None => vec![]
        };

        Environment {
            paths: paths,
            vars: vars,
            directory_stack: Vec::new(),

            // TODO something better than '/'?
            working_directory: env::current_dir().unwrap_or(PathBuf::from("/")),
        }
    }

    ///
    ///
    pub fn from_existing_env() -> Environment {
        let vars = env::vars();
        let capacity = vars.size_hint().1.unwrap_or(0);
        Self::new(vars.fold(
            HashMap::with_capacity(capacity),
            |mut result, (name, value)| {
                result.insert(name, value);
                result
            }
        ))
    }

    /// Sets the current working directory.
    ///
    pub fn set_working_directory(&mut self, path: PathBuf) {
        self.working_directory = path;
    }

    /// Returns the current working directory.
    ///
    pub fn working_directory(&self) -> &PathBuf {
        &self.working_directory
    }

    /// Push directory onto the directory stack.
    ///
    pub fn push_directory(&mut self, path: PathBuf) {
        self.directory_stack.push(path)
    }

    /// Pop a directory from the directory stack.
    ///
    pub fn pop_directory(&mut self) -> Option<PathBuf> {
        self.directory_stack.pop()
    }

    /// Pop a directory from the directory stack.
    ///
    pub fn directory_stack(&self) -> &Vec<PathBuf> {
        &self.directory_stack
    }

    /// Gets the value of a variable from this environment.
    ///
    pub fn var<S: Borrow<String>>(&self, name: &S) -> Option<String> {
        match self.vars.get(name.borrow()) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    /// Gets the value of the `PATH` variable as a `Vec`.
    ///
    pub fn paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    /// Gets the value of a variable from this environment.
    ///
    pub fn vars(&self) -> HashMap<String, String> {
        self.vars.clone()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uses_vars_from_given_map() {
        let expected_paths = vec!["/bin", "/usr/bin", "/usr/local/bin"];
        let joined_paths = env::join_paths(&expected_paths).unwrap().into_string().unwrap();

        let mut vars = HashMap::new();
        vars.insert("PATH".to_owned(), joined_paths.clone());
        vars.insert("HOME".to_owned(), "/Users/foo".to_owned());

        let env = Environment::new(vars);
        assert_eq!(expected_paths, env.paths.iter().map(|v| v.as_os_str()).collect::<Vec<_>>());
        assert_eq!(Some(joined_paths), env.var(&"PATH".to_owned()));
        assert_eq!(Some("/Users/foo".to_owned()), env.var(&"HOME".to_owned()));
        assert_eq!(None, env.var(&"TMPDIR".to_owned()));
    }
}
