use readline::{
    Error,
    Reader
};
use rustyline::{
    Editor,
    error::ReadlineError,
};

pub struct RustylineReader {
    editor: Editor<()>,
}

impl RustylineReader {
    pub fn new() -> RustylineReader {
        RustylineReader {
            editor: Editor::new(),
        }
    }
}

impl Reader for RustylineReader {
    fn get(&mut self) -> Result<String, Error> {
        let line = self.editor.readline(">> ")?;
        self.editor.add_history_entry(&line);
        Ok(line)
    }
}

impl From<ReadlineError> for Error {
    fn from(err: ReadlineError) -> Error {
        match err {
            ReadlineError::Io(io_err) => Error::Io(io_err),
            ReadlineError::Eof => Error::Eof(),
            ReadlineError::Interrupted => Error::Interrupted(),
            _ => Error::Other(),
        }
    }
}
