#![allow(missing_docs)]

pub(crate) mod nul;
pub(crate) mod rar;

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub use rar::Error as RarError;
pub use nul::Error as NulError;

#[derive(Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    #[error(transparent)]
    RarError(#[from] rar::Error),
    #[error(transparent)]
    NulError(#[from] nul::Error),
}

impl<C> From<widestring::error::ContainsNul<C>> for Error {
    fn from(e: widestring::error::ContainsNul<C>) -> Error {
        nul::Error(e.nul_position()).into()
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Error {
        nul::Error(e.nul_position()).into()
    }
}
