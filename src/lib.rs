use unrar_sys as native;
pub use archive::Archive;
pub mod archive;
pub mod error;
mod open_archive;
pub use open_archive::{
    CursorBeforeFile, CursorBeforeHeader, Extract, List, ListSplit, OpenArchive, VolumeInfo
};
