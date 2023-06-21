#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub use archive::Archive;
use unrar_sys as native;
mod archive;
pub mod error;
mod open_archive;
pub use error::UnrarResult;
pub use open_archive::{
    CursorBeforeFile, CursorBeforeHeader, FileHeader, List, ListSplit, OpenArchive, Process,
    VolumeInfo,
};
