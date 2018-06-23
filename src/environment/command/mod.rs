//! Executing commands in an environment.
//!
mod executable;
mod path;

pub use self::{
    executable::{
        Command,
        Error,
        ExitStatus,
    },
    path::Executable,
};
