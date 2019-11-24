extern crate tempdir;
extern crate unrar;

use unrar::Archive;
use unrar::error::{Code, When};
use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[test]
fn list() {
    // No password needed in order to list contents
    let mut entries = Archive::new("data/crypted.rar").list().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, PathBuf::from(".gitignore"));
}

#[test]
fn no_password() {
    let t = TempDir::new("unrar").unwrap();
    let mut arc = Archive::new("data/crypted.rar")
        .extract_to(t.path())
        .unwrap();
    let err = arc.next().unwrap().unwrap_err();
    assert_eq!(err.code, Code::MissingPassword);
    assert_eq!(err.when, When::Process);
}

#[test]
fn version_cat() {
    let t = TempDir::new("unrar").unwrap();
    Archive::with_password("data/crypted.rar", "unrar")
        .extract_to(t.path())
        .unwrap()
        .process()
        .unwrap();
    let mut file = File::open(t.path().join(".gitignore")).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    assert_eq!(s, "target\nCargo.lock\n");
}
