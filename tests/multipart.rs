extern crate unrar;

use std::path::PathBuf;
use unrar::Archive;

#[test]
fn list_missing_volume() {
    let expected = [
        "build.rs",
        "Cargo.toml",
        "examples/lister.rs",
        "src/lib.rs",
        "vendor/unrar/acknow.txt",
        "vendor/unrar/arccmt.cpp",
    ];
    let mut archive = Archive::new("data/archive.part1.rar")
        .open_for_listing()
        .unwrap();
    for (expected, actual) in expected.into_iter().zip(archive.by_ref()) {
        assert_eq!(actual.unwrap().filename, PathBuf::from(expected));
    }
    let data = archive.next().unwrap().unwrap();
    assert_eq!(data.filename, PathBuf::from("vendor/unrar/archive.cpp"));
    println!("{:?}", archive.next().unwrap());
    // assert_eq!(
    //     data.next_volume.map(PathBuf::from),
    //     Some(PathBuf::from("data/archive.part2.rar"))
    // );
}
