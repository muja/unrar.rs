use thiserror::Error;

#[derive(PartialEq, Eq, Error, Debug, Clone, Copy)]
#[error("unexpected NUL at {0}")]
pub struct Error(pub usize);
