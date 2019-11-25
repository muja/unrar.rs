extern crate unrar_sys as native;
extern crate regex;
extern crate libc;
extern crate num;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate bitflags;

pub use archive::Archive;
pub mod error;
pub mod archive;
