#[macro_use]
extern crate nom;
extern crate rustyline;

mod environment;
mod parser;
mod prompt;
mod shell;
mod strings;

use std::process;

fn main() {
  let mut my_shell = shell::Shell {
    prompt: prompt::rustyline::RustylinePrompt::new(),
    parser: parser::GeshlParser::new(),
  };

  if let Err(error) = my_shell.run() {
    println!("error: {:?}", error);
    process::exit(1);
  }
}
