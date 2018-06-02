use std::io;
use std::process;

mod input_loop;

fn main() {
  let stdin = io::stdin();
  let handle = stdin.lock();

  if let Err(error) = input_loop::run(handle) {
    println!("error: {}", error);
    process::exit(1);
  }
}
