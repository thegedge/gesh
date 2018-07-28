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
    working_directory: PathBuf,
    directory_stack: Vec<PathBuf>,

    vars: HashMap<String, String>,
    exported_vars: HashMap<String, String>,
}

impl Environment {
    /// Creates an empty environment.
    ///
    /// Defaults to containing the same paths as the shell's PATH environment variable.
    ///
    pub fn empty() -> Environment {
        Environment {
            paths: Vec::new(),
            vars: HashMap::new(),
            exported_vars: HashMap::new(),
            directory_stack: Vec::new(),

            // TODO something better than '/'?
            working_directory: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }

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
            paths,
            vars: vars.clone(),
            exported_vars: vars,
            directory_stack: Vec::new(),

            // TODO something better than '/'?
            working_directory: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }

    /// Create an environment from the existing environment.
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
        self.vars.insert("PWD".to_owned(), String::from(path.to_string_lossy()));
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

    /// Gets the value of the `PATH` variable as a `Vec`.
    ///
    pub fn paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    /// Gets the value of a variable from this environment.
    ///
    pub fn get<S: Borrow<String>>(&self, name: &S) -> Option<String> {
        match self.vars.get(name.borrow()) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    /// Sets a variable in the environment.
    ///
    pub fn set(&mut self, name: String, value: String) -> Option<String> {
        self.vars.insert(name, value)
    }

    /// Exports a varable.
    ///
    /// Exported variables will be included in the environment of subsequent commands. Variables
    /// that haven't been exported can still be used in the shell.
    ///
    pub fn export(&mut self, name: String) {
        if let Some(val) = self.get(&name) {
            self.exported_vars.insert(name, val);
        }
    }

    /// Gets the exported variables in this environment.
    ///
    /// Exported variables are the ones that get used in subsequent commands.
    ///
    pub fn exported_vars(&self) -> HashMap<String, String> {
        self.exported_vars.clone()
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
        assert_eq!(Some(joined_paths), env.get(&"PATH".to_owned()));
        assert_eq!(Some("/Users/foo".to_owned()), env.get(&"HOME".to_owned()));
        assert_eq!(None, env.get(&"TMPDIR".to_owned()));
    }

    #[test]
    fn test_push_and_pop() {
        let mut env = Environment::new(HashMap::new());
        env.push_directory(PathBuf::from("/usr"));
        env.push_directory(PathBuf::from("/usr/local"));
        env.push_directory(PathBuf::from("/usr/local/bin"));

        assert_eq!(Some(PathBuf::from("/usr/local/bin")), env.pop_directory());

        assert_eq!(
            &vec![PathBuf::from("/usr"), PathBuf::from("/usr/local")],
            env.directory_stack()
        );

        assert_eq!(Some(PathBuf::from("/usr/local")), env.pop_directory());
        assert_eq!(Some(PathBuf::from("/usr/")), env.pop_directory());
        assert_eq!(None, env.pop_directory());
    }

    #[test]
    fn test_export_moves_variable_to_exported_variables() {
        let mut env = Environment::new(HashMap::new());
        env.set("FOO".to_owned(), "fooval".to_owned());
        env.set("BAR".to_owned(), "barval".to_owned());
        env.export("FOO".to_owned());

        assert_eq!(Some(&"fooval".to_owned()), env.exported_vars().get(&"FOO".to_owned()));
        assert_eq!(None, env.exported_vars().get(&"BAR".to_owned()));
    }
}
