#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod archive;
mod open_archive;
pub mod error;

pub use error::Error;
pub use error::RarError;
pub use error::Result;
pub use archive::Archive;
pub use open_archive::{
    CursorBeforeFile, CursorBeforeHeader, FileHeader, List, ListSplit, OpenArchive, Process,
    VolumeInfo,
};
