# unrar.rs

[![Travis Build Status](https://travis-ci.org/muja/unrar.rs.png)](https://travis-ci.org/muja/unrar.rs)

Please look inside the [examples directory](./examples) to see how to use this library!

Basic example to list archive entries:

```rust
extern crate unrar;

fn main() {
    for entry in unrar::Archive::new("archive.rar").list().unwrap() {
        println!("{}", entry.unwrap().filename);
    }
}
```

Run this example using: `cargo run --example basic_list`.  
Note that you have to put a `archive.rar` in this directory first.  
For example, using the `rar` CLI: `rar a archive.rar .`

# Contributing

Feel free to contribute! If you detect a bug or issue, open an issue.

Pull requests are welcome!

# Help

If you need help using the library, ping me at irc.mozilla.org, my handle is **danyel**
