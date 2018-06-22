//! Executing commands in an environment.
//!
mod executable;
mod path;

pub use self::{
    executable::{
        Error,
        ExecutableUnit,
        ExitStatus,
    },
    path::Command,
};
