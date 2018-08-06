//! Encapsulates the core components of a shell.
//!
//! This inclues:
//! * Input
//! * Command parsing
//! * Job management
//!
use command::{
    self,
    Context,
    ExitStatus,
    Registry,
};

use environment::{
    Environment,
};

use geshl::{
    self,

    Command,
    ParsedLine,
    SetVariable,
};

use prompt::{
    self,
    Prompt,
};

use strings;

use std::{
    env,
};

/// A user shell.
///
pub struct Shell<R: Prompt> {
    pub prompt: R,
    pub parser: geshl::Parser,
}

/// Enumeration of all possible errors that can occur in the shell.
///
#[derive(Debug)]
pub enum Error {
    CommandError(command::Error),
    VarError(env::VarError),
    ParserError(geshl::Error),
    PromptError(prompt::Error),
}

impl<R: Prompt> Shell<R> {
    /// Runs the shell's main read -> parse -> execute loop.
    ///
    pub fn run(&mut self) -> Result<ExitStatus, Error> {
        let mut env = Environment::from_existing_env();
        let registry = Registry::for_env(&env);

        loop {
            self.prompt.set_prompt(env.working_directory().to_string_lossy().into_owned().to_string() + "$ ");

            let parsed_line = match self.prompt.get() {
                Ok(raw_line) => self.parser.parse(raw_line)?,
                Err(prompt::Error::Eof()) => break,
                Err(prompt::Error::Interrupted()) => continue,
                Err(err) => return Err(Error::PromptError(err)),
            };

            match parsed_line {
                ParsedLine::Command(Command { vars, args: pieces }) => {
                    // First, process the pieces
                    let mut args = strings::to_string_vec(pieces.into_iter(), &env);
                    if args.is_empty() {
                        continue
                    }

                    let cmd = args.remove(0);

                    // If there are variables, we need to create a temporary environment with
                    // the new vars. Otherwise we can just use the current one.
                    let result = if vars.is_empty() {
                        registry.execute(&cmd, Context { env: &mut env, args })
                    } else {
                        let mut temp_env = env.clone();
                        for SetVariable { name, value } in vars {
                            let interpolated_value = strings::shellstring_to_string(&value, &temp_env);
                            temp_env.set(name.clone(), interpolated_value);
                            temp_env.export(name);
                        }

                        registry.execute(&cmd, Context { env: &mut temp_env, args })
                    };

                    if let Ok(ExitStatus::ExitWith(code)) = result {
                        return Ok(ExitStatus::ExitWith(code));
                    };
                },

                ParsedLine::SetVariables(vars) => {
                    for SetVariable { name, value } in vars {
                        let interpolated_value = strings::shellstring_to_string(&value, &env);
                        env.set(name, interpolated_value);
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

impl From<geshl::Error> for Error {
    fn from(err: geshl::Error) -> Self {
        Error::ParserError(err)
    }
}

impl From<prompt::Error> for Error {
    fn from(err: prompt::Error) -> Self {
        Error::PromptError(err)
    }
}
