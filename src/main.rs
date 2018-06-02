extern crate rustyline;

mod shell;
mod readline;

use std::process;
use readline::rustyline::RustylineReader;

fn main() {
  let mut my_shell = shell::Shell {
    reader: RustylineReader::new(),
  };

  if let Err(error) = my_shell.run() {
    println!("error: {:?}", error);
    process::exit(1);
  }
}
