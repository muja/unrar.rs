extern crate unrar;

use std::path::PathBuf;
use unrar::error::{Code, When};
use unrar::Archive;

#[test]
fn list_missing_volume() {
    let expected: Vec<PathBuf> = vec![
        "build.rs",
        "Cargo.toml",
        "examples/lister.rs",
        "src/lib.rs",
        "vendor/unrar/acknow.txt",
        "vendor/unrar/arccmt.cpp",
    ]
    .iter()
    .map(|x| x.into())
    .collect();
    let mut archive = Archive::new("data/archive.part1.rar")
        .unwrap()
        .list()
        .unwrap();
    for (i, e) in archive.by_ref().enumerate().take(expected.len()) {
        assert_eq!(e.unwrap().filename, expected[i]);
    }
    let err = archive.next().unwrap().err().unwrap();
    assert_eq!(err.code, Code::EOpen);
    assert_eq!(err.when, When::Process);
    let data = err.data.unwrap();
    assert_eq!(data.filename, PathBuf::from("vendor/unrar/archive.cpp"));
    assert_eq!(
        PathBuf::from(data.next_volume.unwrap()),
        PathBuf::from("data/archive.part2.rar")
    );
}
