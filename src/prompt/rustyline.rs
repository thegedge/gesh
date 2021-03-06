use super::{
    Error,
    Prompt
};

use rustyline::{
    Editor,
    error::ReadlineError,
};

pub struct RustylinePrompt {
    editor: Editor<()>,
    prompt: String,
}

impl RustylinePrompt {
    pub fn new() -> RustylinePrompt {
        RustylinePrompt {
            editor: Editor::new(),
            prompt: "gesh$ ".to_owned(),
        }
    }
}

impl Prompt for RustylinePrompt {
    fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    fn get(&mut self) -> Result<String, Error> {
        let line = self.editor.readline(&self.prompt)?;
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
