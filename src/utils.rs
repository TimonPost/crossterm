//! # Utils

pub use self::{
    command::{Command, ExecutableCommand, Output, QueueableCommand},
    error::{ErrorKind, Result},
};

mod command;
mod error;
pub(crate) mod functions;
pub(crate) mod macros;
pub(crate) mod sys;
