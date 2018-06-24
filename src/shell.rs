//! Encapsulates the core components of a shell.
//!
//! This inclues:
//! * Input
//! * Command parsing
//! * Job management
//!
use command::{
    self,
    ExitStatus,
    Registry,
};

use environment::{
    Environment,
};

use parser::{
    self,
    ParsedLine,
    Parser,
};

use prompt::{
    self,
    Prompt,
};

use std::{
    env,
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
    CommandError(command::Error),
    VarError(env::VarError),
    ParserError(parser::Error),
    PromptError(prompt::Error),
}

impl<R: Prompt, P: Parser> Shell<R, P> {
    /// Runs the shell's main read -> parse -> execute loop.
    ///
    pub fn run(&mut self) -> Result<ExitStatus, Error> {
        let mut env = Environment::from_existing_env();
        let registry = Registry::new();

        loop {
            self.prompt.set_prompt(env.working_directory().to_string_lossy().into_owned().to_string() + "$ ");

            let parsed_line = match self.prompt.get() {
                Ok(raw_line) => self.parser.parse(raw_line)?,
                Err(prompt::Error::Eof()) => break,
                Err(prompt::Error::Interrupted()) => continue,
                Err(err) => return Err(Error::PromptError(err)),
            };

            match parsed_line {
                ParsedLine::Command(cmd, args) => {
                    match cmd.to_string(&env) {
                        Some(interpolated_cmd) => {
                            let result = registry.execute(&mut env, &interpolated_cmd, args);
                            if let Ok(ExitStatus::ExitWith(code)) = result {
                                return Ok(ExitStatus::ExitWith(code));
                            };
                        },
                        None => println!("No command given!")
                    }
                },
                ParsedLine::Empty => continue,
            }
        }
        Ok(ExitStatus::Success(0))
    }
}

impl From<command::Error> for Error {
    fn from(err: command::Error) -> Self {
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
