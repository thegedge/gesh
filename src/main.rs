use std::io::{self, BufRead};
use std::process;

fn main() {
  let stdin = io::stdin();
  let mut handle = stdin.lock();
  let mut command_string = String::new();

  loop {
    match handle.read_line(&mut command_string) {
      Ok(_) => {
        print!("{}", command_string);
      }
      Err(err) => {
        println!("error: {}", err);
        process::exit(1);
      }
    }
  };
}
