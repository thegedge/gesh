extern crate rustyline;

use std::process;

mod input_loop;

fn main() {
  if let Err(error) = input_loop::run() {
    println!("error: {}", error);
    process::exit(1);
  }
}
