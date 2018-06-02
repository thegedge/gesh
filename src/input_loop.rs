use rustyline::error::ReadlineError;
use rustyline::Editor;

/// Executes an input loop for the shell.
///
/// Input is received via a readline-like UI, and should provide command history, completion,
/// vi/emacs modes, and all the other bells and whistles that people expect out of their shell.
///
/// # Errors
///
/// Propagates any input errors received when reading
///
pub fn run() -> Result<(), ReadlineError> {
    let mut editor = Editor::<()>::new();

    if let Err(_) = editor.load_history("history.txt") {
        // TODO write to a log file
    }

    loop {
        match editor.readline(">> ") {
            Ok(line) => {
                editor.add_history_entry(&line);
                println!("{}", line);
            },
            Err(ReadlineError::Interrupted) => {
                continue
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                return Err(err)
            }
        }
    };

    if let Err(_) = editor.save_history("history.txt") {
        // TODO write to a log file
    }

    Ok(())
}
