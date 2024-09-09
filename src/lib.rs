#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod archive;
mod error;
mod pathed;
mod open_archive;

pub use error::*;
pub use pathed::Nulable;

pub use archive::Archive;
pub use open_archive::{
    CursorBeforeFile, CursorBeforeHeader, FileHeader, List, ListSplit, OpenArchive, Process,
    VolumeInfo,
};
