//! Encapsulates the core components of a shell.
//!
//! This inclues:
//! * Input
//! * Command parsing
//! * Job management
//!
use std::{
    env,
};

use prompt::{
    self,
    Prompt,
};

use parser::{
    self,
    ParsedLine,
    Parser,
};

use environment::{
    self,
    Environment,
};

/// A user shell.
///
pub struct Shell<R: Prompt, P: Parser> {
    pub prompt: R,
    pub parser: P,
}

/// Enumeration of all possible errors that can occur in the shell.
///
#[derive(Debug)]
pub enum Error {
    CommandError(environment::CommandError),
    VarError(env::VarError),
    ParserError(parser::Error),
    PromptError(prompt::Error),
}

impl<R: Prompt, P: Parser> Shell<R, P> {
    /// Runs the shell's main read -> parse -> execute loop.
    ///
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match env::current_dir() {
                Ok(dir) => self.prompt.set_prompt(dir.to_string_lossy().into_owned().to_string() + "$ "),
                Err(_) => self.prompt.set_prompt("gesh$ ".to_owned()),
            };

            let parsed_line = match self.prompt.get() {
                Ok(raw_line) => self.parser.parse(raw_line)?,
                Err(prompt::Error::Eof()) => break,
                Err(prompt::Error::Interrupted()) => continue,
                Err(err) => return Err(Error::PromptError(err)),
            };

            match parsed_line {
                ParsedLine::Command(cmd, args) => {
                    let env = Environment::from_existing_env();
                    match cmd.to_string(&env) {
                        Some(interpolated_cmd) => {
                            let result = env.execute(interpolated_cmd, args);
                            println!("{:?}", result);
                        },
                        None => println!("No command given!")
                    }
                },
                ParsedLine::Empty => continue,
            }
        }
        Ok(())
    }
}

impl From<environment::CommandError> for Error {
    fn from(err: environment::CommandError) -> Self {
        Error::CommandError(err)
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Self {
        Error::VarError(err)
    }
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Self {
        Error::ParserError(err)
    }
}

impl From<prompt::Error> for Error {
    fn from(err: prompt::Error) -> Self {
        Error::PromptError(err)
    }
}
