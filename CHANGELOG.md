# Changelog

All notable changes to this project will be documented in this file.

## [0.5.3] - 2024-02-22

### Documentation

-  vendor: htm -> md (for viewing in browser)

### Features

-  *: add test mode

### Miscellaneous Tasks

-  unrar_sys: add Apache2 license to Cargo.toml

### Styling

-  docs: fix markdown

## [0.5.2] - 2023-11-11

Release 0.5.2 fixes the build for Windows targets.
Also adds other minor improvements (docs, perf).

### Bug Fixes

-  archive: dont strip parents in path methods
-  unrar_sys: remove indirect dependency to MSVC comsupp library for windows-gnu target
-  unrar_sys: use winapi crate for all windows targets

### Documentation

-  archive: add docs examples for Archive::break_open
-  unrar_sys: add vendor documentation

### Miscellaneous Tasks

-  unrar_sys: link against static libstdc++ on windows-gnu targets
-  unrar_sys: fall back to C++14 (minimal version supported by MSVC)
-  unrar_sys: upgrade DLL version to 6.24.0
-  unrar_sys: add upgrade instructions and script

### Performance

-  archive: use as_str instead of to_string_lossy where sensible

### Testing

-  *: use PathBuf to not fail on windows

## [0.5.1] - 2023-06-28

### Bug Fixes

-  open_archive: fix NULL deref, pass valid pointer
-  *: a NULL pointer dereference caused undefined behavior in the callback

### Example

-  unrar_sys: format lister example
-  unrar_sys: fix windows build for lister example

### Miscellaneous Tasks

-  *: add test step for unrar_sys library

## [0.5.0] - 2023-06-22

### Bug Fixes

-  unrar_sys: fix broken code in example and test
-  *: avoid endlessly returning errors in Iterator

### Documentation

-  *: further improve crate-level docs

### Example

-  read_named: rename example and print content

### Features

-  *: implement typestate pattern, completely rewrite major parts
-  *: upgrade dependencies
-  *: Archive::as_first_part returns self
-  *: add force_heal method if eagerly returning None is not desired

### Miscellaneous Tasks

-  *: update author e-mail
-  *: update authors
-  *: add Github Actions workflow
-  dll: upgrade DLL version to 6.2.8

### Refactor

-  *: edition=2021, remove superfluous extern crates
-  *: edition=2021, remove superfluous extern crates
-  *: return Result<Option<T>,E> instead of Option<Result<T,E>>

### Styling

-  *: cargo fmt
-  *: use std::ptr::null/_mut instead of 0 as *_
