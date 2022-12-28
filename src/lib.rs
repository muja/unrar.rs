extern crate num;
extern crate regex;
extern crate unrar_sys as native;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate bitflags;
extern crate widestring;

pub use archive::Archive;
pub mod archive;
pub mod error;
mod open_archive;
pub use open_archive::{
    CursorBeforeFile, CursorBeforeHeader, Extract, List, ListSplit, OpenArchive, VolumeInfo
};
