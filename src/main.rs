#[macro_use]
extern crate nom;
extern crate rustyline;

mod environment;
mod parser;
mod prompt;
mod shell;
mod strings;

use std::process;
use environment::ExitStatus;

fn main() {
  let mut my_shell = shell::Shell {
    prompt: prompt::rustyline::RustylinePrompt::new(),
    parser: parser::GeshlParser::new(),
  };

  match my_shell.run() {
    Ok(ExitStatus::Success(status)) => process::exit(status),
    Err(error) => {
      println!("error: {:?}", error);
      process::exit(1);
    }
  }
}
