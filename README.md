# unrar.rs

[![Travis Build Status](https://travis-ci.org/muja/unrar.rs.svg)](https://travis-ci.org/muja/unrar.rs)
[![crates.io](http://meritbadge.herokuapp.com/unrar)](https://crates.io/crates/unrar)

High-level wrapper around the unrar C library provided by [rarlab](http://rarlab.com).

Please look inside the [examples directory](./examples) to see how to use this library!  
Specifically the [**lister**](./examples/lister.rs) example is well documented and advanced!

Basic example to list archive entries:

```rust
extern crate unrar;

fn main() {
    for entry in unrar::Archive::new("archive.rar".into()).list().unwrap() {
        println!("{}", entry.unwrap());
    }
}
```

Run this example: `cargo run --example basic_list`.  
Note that you need to put an `archive.rar` in the directory first.  
For example, by using the `rar` CLI: `rar a archive.rar .`

# Features

- [x] Multipart files
- [x] Listing archives
- [x] Extracting them
- [x] Testing them (not fully tested yet)
- [x] Encrypted archives with password
- [x] Linked statically against the unrar source.
- [x] Build unrar C++ code from source
- [x] Basic functionality that operates on filenames / paths (without reading archives)
- [ ] More documentation / RustDoc
- [ ] Tests

# Contributing

Feel free to contribute! If you detect a bug or issue, open an issue.

Pull requests are welcome!

# Help

If you need help using the library, ping me at irc.mozilla.org, my handle is **danyel**
