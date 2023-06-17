//! The unrar crate can inspect and extract RAR archives.
//!
//! # Overview
//!
//! The primary type in this crate is [`Archive`](struct.Archive.html)
//! which denotes an archive on the file system. `Archive` itself makes no
//! FS operations, unless one of the `open` methods are called, returning
//! an [`OpenArchive`](struct.OpenArchive.html).
//!
//! # Archive
//!
//! The [`Archive`](struct.Archive.html) struct provides two major classes of methods:
//!
//!    1. methods that do not touch the FS. These are opinionated utility methods
//!         that are based on RAR path conventions out in the wild. Most commonly, multipart
//!         files usually have extensions such as `.part08.rar` or `.r08.rar`. Since extracting
//!         must start at the first part, it may be helpful to figure that out using, for instance,
//!         [`archive.as_first_part()`](Archive::as_first_part)
//!    2. methods that open the underlying path in the specified mode
//!         (possible modes are [`List`], [`ListSplit`] and [`Process`]).
//!         These methods have the word `open` in them, are fallible operations,
//!         return [`OpenArchive`](struct.OpenArchive.html) inside a `Result` and are as follows:
//!         - [`open_for_listing`](Archive::open_for_listing) and
//!             [`open_for_listing_split`](Archive::open_for_listing_split): list the archive
//!             entries (skipping over content/payload)
//!         - [`open_for_processing`](Archive::open_for_processing): process archive entries
//!             as well as content/payload
//!         - [`break_open`](Archive::break_open): read archive even if an error is returned,
//!             if possible. The [`OpenMode`](open_archive/struct.OpenMode.html) must be provided
//!             explicitly.
//!
//! # OpenArchive
//! An archive is opened in one of these three modes: [`List`], [`ListSplit`] or [`Process`].
//! This library does not provide random access into archives. Instead, files inside the archive
//! can only be processed as a stream, unidirectionally, front to back, alternating between
//! [`ReadHeader`] and [`ProcessFile`] operations (as dictated by the underlying C++ library).  
//!
//! That is the idea behind cursors:
//!
//! ## OpenArchive: Cursors
//!
//! Via cursors, the archive keeps track what operation is permitted next:
//!    - [`CursorBeforeHeader`] -> [`ReadHeader`]
//!    - [`CursorBeforeFile`] -> [`ProcessFile`]
//!
//! The library enforces this by making
//! use of the [typestate pattern](https://cliffle.com/blog/rust-typestate/). An archive, once
//! opened, starts in the `CursorBeforeHeader` state and, thus, must have its [`read_header`] method
//! called, which returns a new `OpenArchive` instance in the `CursorBeforeFile` state that only
//! exposes methods that internally map to the `ProcessFile` operation.
//! Which methods are accessible in each step depends on the archive's current state and the
//! mode it was opened in.
//!
//! ## Available methods for Open mode/Cursor position combinations
//! Here is an overview of what methods are exposed for the OpenMode/Cursor combinations:
//!
//! | Open mode↓ ╲ Cursor position→| before header   | before file                                                            |
//! |------------------------------|-----------------|------------------------------------------------------------------------|
//! | [`List`], [`ListSplit`]      | [`read_header`] | [`skip`]                                                               |
//! | [`Process`]                  | [`read_header`] | [`skip`], [`read`], [`extract`], [`extract_to`], [`extract_with_base`] |
//!
//! ## OpenArchive: Iterator
//!
//! Archives opened in [`List`] or [`ListSplit`] mode also implement [`Iterator`] whereas archives in
//! [`Process`] mode do not (though this may change in future releases). That is because the first
//! two will read and return headers while being forced to skip over the payload whereas the latter
//! has more sophisticated processing possibilities that's not easy to convey using an [`Iterator`].
//!
//! [`read_header`]: OpenArchive::read_header
//! [`skip`]: OpenArchive::skip
//! [`read`]: OpenArchive::read
//! [`extract`]: OpenArchive::extract
//! [`extract_to`]: OpenArchive::extract_to
//! [`extract_with_base`]: OpenArchive::extract_with_base
//! [`ReadHeader`]: unrar_sys::RARReadHeaderEx
//! [`ProcessFile`]: unrar_sys::RARProcessFileW

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
