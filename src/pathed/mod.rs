#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(not(target_os = "linux"), path = "all.rs")]
mod os;

pub(crate) use os::*;
use thiserror::Error;

/// Error for functions that convert to an ffi String representation
/// before calling into the ffi.
#[derive(Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Nulable<E: crate::RarError> {
    /// Underlying ffi error, meaning that conversion was successful
    /// but the ffi call failed
    #[error(transparent)]
    Rar(#[from] E),
    /// Conversion error due to a NUL character in the String
    #[error("unexpected NUL at {0}")]
    Nul(usize),
}

impl<E: crate::RarError> Nulable<E> {
    /// returns the contained ffi error, consuming the `self` value
    /// 
    /// # Panics
    /// 
    /// Panics if the `self` value is `Nul`
    /// 
    pub fn unwrap(self) -> E {
        match self {
            Nulable::Rar(e) => e,
            nul => panic!("{nul}"),
        }
    }
}

impl<C, E: crate::RarError> From<widestring::error::ContainsNul<C>> for Nulable<E> {
    fn from(e: widestring::error::ContainsNul<C>) -> Self {
        Nulable::Nul(e.nul_position())
    }
}

impl<E: crate::RarError> From<std::ffi::NulError> for Nulable<E> {
    fn from(e: std::ffi::NulError) -> Self {
        Nulable::Nul(e.nul_position())
    }
}
