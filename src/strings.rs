//! Provides string types that can be interpolated within an environment.
//!
use glob;

use std::{
    ops,
};

use super::{
    environment::Environment,
};

/// A string in a shell, which can have many components consisting of:
///
/// - fixed string components,
/// - variable interpolations, and
/// - path components (for example, `~` is the user's home directory).
///
#[derive(Clone, Debug, PartialEq)]
pub struct ShellString {
    pieces: Vec<Piece>,
}

/// A component of an interpolated string
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    /// A fixed string.
    Fixed(String),

    /// A glob string.
    ///
    /// Globs can be the following:
    /// - `?`, to match a single character,
    /// - `*`, to match zero or more characters,
    /// - `**`, to match the current directory or arbitrary subdirectories,
    /// - `[...]`, to match any character within the square brackets, or
    /// - `[!...]`, to match any character not within the square brackets.
    Glob(String),

    /// A shell variable.
    Variable(String),
}

/// Converts a list of `ShellString`s to a list of `String`s.
///
pub fn to_string_vec<V>(values: V, env: &Environment) -> Vec<String>
    where V: Iterator<Item = ShellString>
{
    values.fold(Vec::new(), |mut acc, string| {
        if string.has_glob() {
            let paths = glob::glob_with(
                &string.to_string(env),
                &glob::MatchOptions {
                    case_sensitive: true,
                    require_literal_separator: true,
                    require_literal_leading_dot: true,
                }
            );

            if paths.is_err() {
                return Vec::new();
            }

            acc.extend(
                paths.unwrap()
                    .filter_map(|path| path.ok())
                    .filter_map(|path| path.into_os_string().into_string().ok())
                    .collect::<Vec<_>>()
            )
        } else {
            acc.push(string.to_string(env));
        }

        acc
    })
}

impl ShellString {
    /// Returns whether or not there is a glob component in this `ShellString`
    ///
    pub fn has_glob(&self) -> bool {
        self.pieces.iter().any(|v| {
            if let Piece::Glob(_) = v {
                true
            } else {
                false
            }
        })
    }

    /// Converts this shell string into a regular string.
    ///
    /// Path and variable interpolations are done via the given `Environment`, as specified
    /// in `Piece::to_string`.
    ///
    pub fn to_string(&self, env: &Environment) -> String {
        self.pieces.iter().map(|piece| piece.to_string(env)).collect()
    }
}

impl ops::Add<ShellString> for ShellString {
    type Output = ShellString;

    fn add(mut self, rhs: ShellString) -> Self::Output {
        self.pieces.extend(rhs.pieces.into_iter());
        ShellString {
            pieces: self.pieces
        }
    }
}

impl <'a> From<&'a str> for ShellString {
    fn from(value: &str) -> Self {
        ShellString { pieces: vec![Piece::Fixed(value.to_owned())] }
    }
}

impl From<String> for ShellString {
    fn from(value: String) -> Self {
        ShellString { pieces: vec![Piece::Fixed(value)] }
    }
}

impl From<Piece> for ShellString {
    fn from(value: Piece) -> Self {
        ShellString { pieces: vec![value] }
    }
}

impl From<Vec<Piece>> for ShellString {
    fn from(value: Vec<Piece>) -> Self {
        ShellString { pieces: value }
    }
}

impl Piece {
    /// Converts this piece into a `String` with a given environment.
    ///
    /// If the variable referenced in `Piece::Variable` isn't in the environment, it will be
    /// substituted with an empty string.
    ///
    pub fn to_string(&self, env: &Environment) -> String {
        match &self {
            Piece::Fixed(ref s) => s.clone(),
            Piece::Glob(ref s) => s.clone(),
            Piece::Variable(ref name) => env.get(&name).unwrap_or_else(|| "".to_owned()),
        }
    }
}

impl <'a> From<&'a str> for Piece {
    fn from(value: &'a str) -> Self {
        Piece::Fixed(value.to_owned())
    }
}

impl From<String> for Piece {
    fn from(value: String) -> Self {
        Piece::Fixed(value)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        env,
        ffi::OsStr,
        path::PathBuf,
    };

    use super::*;

    #[test]
    fn test_has_glob_false_with_no_globs() {
        let string = ShellString::from(vec![
            Piece::from(" is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        assert_eq!(false, string.has_glob());
    }

    #[test]
    fn test_has_glob_false_with_globs() {
        let string = ShellString::from(vec![
            Piece::from(" is a "),
            Piece::Glob("*".to_owned()),
            Piece::Variable("WHAT".to_owned()),
        ]);

        assert_eq!(true, string.has_glob());
    }

    #[test]
    fn test_to_string_returns_string_when_var_exists() {
        let shell_string = ShellString::from(vec![
            Piece::from("this is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        let mut vars = HashMap::new();
        vars.insert("WHAT".to_owned(), "test".to_owned());

        let env = Environment::new(vars);

        assert_eq!("this is a test".to_owned(), shell_string.to_string(&env));
    }

    #[test]
    fn test_to_string_returns_empty_string_when_var_doesnt_exist() {
        let shell_string = ShellString::from(vec![
            Piece::from("this is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        let env = Environment::new(HashMap::new());

        assert_eq!("this is a ".to_owned(), shell_string.to_string(&env));
    }

    #[test]
    fn test_to_string_vec_returns_vec_of_strings() {
        let shell_strings = vec![
            ShellString::from(vec![
                Piece::from("this is a "),
                Piece::Variable("WHAT".to_owned()),
            ]),
            ShellString::from("another"),
        ];

        let mut vars = HashMap::new();
        vars.insert("WHAT".to_owned(), "test".to_owned());

        let env = Environment::new(vars);

        assert_eq!(
            vec!["this is a test".to_owned(), "another".to_owned()],
            to_string_vec(shell_strings.into_iter(), &env)
        );
    }

    #[test]
    fn test_to_string_vec_returns_paths_when_globbed() {
        let shell_strings = vec![
            ShellString::from(Piece::Glob("Cargo*".to_owned()))
        ];

        let mut env = Environment::from_existing_env();
        env.set_working_directory(project_root());

        let mut actual = to_string_vec(shell_strings.into_iter(), &env);
        actual.sort_unstable();

        assert_eq!(
            vec!["Cargo.lock".to_owned(), "Cargo.toml".to_owned()],
            actual
        );
    }

    #[test]
    fn test_adding_shellstrings_concatenates() {
        let string1 = ShellString::from("this");
        let string2 = ShellString::from(vec![
            Piece::from(" is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        let expected = ShellString::from(vec![
            Piece::from("this"),
            Piece::from(" is a "),
            Piece::Variable("WHAT".to_owned()),
        ]);

        assert_eq!(expected, string1 + string2);
    }

    fn project_root() -> PathBuf {
        let bin = env::current_exe().expect("exe path");
        let mut target_dir = PathBuf::from(bin.parent().expect("bin parent"));
        while target_dir.file_name() != Some(OsStr::new("target")) {
            target_dir.pop();
        }
        target_dir.pop();
        target_dir
    }
}
