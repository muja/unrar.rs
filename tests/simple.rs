extern crate tempdir;
extern crate unrar;

use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn version_list() {
    let mut entries = unrar::Archive::new("data/version.rar".into()).list().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, "VERSION");
}

#[test]
fn version_cat() {
    let t = TempDir::new("unrar").unwrap();
    unrar::Archive::new("data/version.rar".into())
        .extract_to(t.path().to_str().unwrap().into())
        .unwrap()
        .process()
        .unwrap();
    let mut file = File::open(t.path().join("VERSION")).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    assert_eq!(s, "unrar-0.4.0");
}
