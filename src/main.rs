// Needed to impl FnMut by command::path::Executable
#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate geshl;
extern crate glob;
extern crate rustyline;

mod command;
mod environment;
mod prompt;
mod shell;
mod strings;

use std::process;
use command::ExitStatus;

fn main() {
  let mut my_shell = shell::Shell {
    prompt: prompt::rustyline::RustylinePrompt::new(),
    parser: geshl::Parser::new(),
  };

  match my_shell.run() {
    Ok(ExitStatus::Success(status)) => process::exit(status as i32),
    Ok(ExitStatus::ExitWith(status)) => process::exit(status as i32),
    Err(error) => {
      println!("error: {:?}", error);
      process::exit(255);
    }
  }
}
