#[macro_use]
extern crate nom;
extern crate rustyline;

mod parser;
mod readline;
mod shell;

use parser::Parser;
use readline::rustyline::RustylineReader;
use std::process;


fn main() {
  let mut my_shell = shell::Shell {
    reader: RustylineReader::new(),
    parser: Parser::new(),
  };

  if let Err(error) = my_shell.run() {
    println!("error: {:?}", error);
    process::exit(1);
  }
}
