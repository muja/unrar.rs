extern crate tempdir;
extern crate unrar;

use unrar::Archive;
use unrar::error::{Code, When};
use std::io::prelude::*;

#[test]
fn list_missing_volume() {
    let expected = vec![
        "build.rs",
        "Cargo.toml",
        "examples/lister.rs",
        "src/lib.rs",
        "vendor/unrar/acknow.txt",
        "vendor/unrar/arccmt.cpp"
    ];
    let mut archive = Archive::new("data/archive.part1.rar".into()).list().unwrap();
    for (i, e) in archive.by_ref().enumerate().take(expected.len()) {
        assert_eq!(e.unwrap().filename, expected[i]);
    }
    let err = archive.next().unwrap().err().unwrap();
    assert_eq!(err.code, Code::EOpen);
    assert_eq!(err.when, When::Process);
    let data = err.data.unwrap();
    assert_eq!(data.filename, "vendor/unrar/archive.cpp");
    assert_eq!(data.next_volume.unwrap(), "data/archive.part2.rar");
}
