# unrar.rs

[![crates.io](https://img.shields.io/crates/v/unrar.svg)](https://crates.io/crates/unrar)
[![API docs](https://docs.rs/unrar/badge.svg)](https://docs.rs/unrar)
[![build](https://github.com/muja/unrar.rs/workflows/ci/badge.svg)](https://github.com/muja/unrar.rs/actions?query=workflow%3Aci)
[![MIT license](https://img.shields.io/badge/license-MIT-blue.svg)](./README.md)

High-level wrapper around the unrar C library provided by [rarlab](http://rarlab.com).

This library can only *extract* and *list* archives, it cannot *create* them.

Please look inside the [examples directory](./examples) to see how to use this library.
Specifically the [**lister**](./examples/lister.rs) example is well documented and advanced!

Basic example to list archive entries:

```rust
use unrar::Archive as RarArchive;

fn main() {
    for entry in RarArchive::new("archive.rar").open_for_listing().unwrap() {
        println!("{}", entry.unwrap());
    }
}
```

Run this example: `cargo run --example basic_list path/to/archive.rar`.
You can create an archive by using the `rar` CLI: `rar a archive.rar .`

# Features

- [x] Multipart files
- [x] Listing archives
- [x] Extracting them
- [x] Reading them into memory (without extracting)
- [x] Testing them
- [x] Encrypted archives with password
- [x] Linked statically against the unrar source.
- [x] Build unrar C++ code from source
- [x] Basic functionality that operates on filenames / paths (without reading archives)
- [x] More documentation / RustDoc
- [x] Tests

# Non-Features

- Creating archives
- Pure Rust implementation

# Contributing

Feel free to contribute! If you detect a bug or issue, open an issue.

Pull requests are welcome!

# Help

If you need help using the library, ping me at irc.mozilla.org, my handle is **danyel**

# License

While this crate uses the MIT license for the Rust parts,
the embedded [C++ library](./unrar_sys/vendor/unrar) has a different license.

For more informations, see its [license file](./unrar_sys/vendor/unrar/license.txt).
