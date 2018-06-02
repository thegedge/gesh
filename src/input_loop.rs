use std::io::{Error, BufRead};

/// Executes an input loop for the shell.
///
/// Reads characters from `input` until EOF is reached.
///
/// # Errors
///
/// Propagates any input errors received when reading from `input`
pub fn run<R: BufRead>(mut input: R) -> Result<(), Error> {
  let mut command_string = String::new();

  loop {
    let _ = input.read_line(&mut command_string)?;
    print!("{}", command_string);
  };
}
