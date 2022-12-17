extern crate tempdir;
extern crate unrar;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use tempdir::TempDir;
use unrar::error::{Code, When};
use unrar::Archive;

#[test]
fn list() {
    // No password needed in order to list contents
    let mut entries = Archive::new("data/crypted.rar").unwrap().list().unwrap();
    assert_eq!(
        entries.next().unwrap().unwrap().filename,
        PathBuf::from(".gitignore")
    );
}

#[test]
fn no_password() {
    let t = TempDir::new("unrar").unwrap();
    let mut arc = Archive::new("data/crypted.rar")
        .unwrap()
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
        .unwrap()
        .extract_to(t.path())
        .unwrap()
        .process()
        .unwrap();
    let mut file = File::open(t.path().join(".gitignore")).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    assert_eq!(s, "target\nCargo.lock\n");
}

#[test]
fn list_encrypted_headers() {
    let mut entries = Archive::with_password("data/comment-hpw-password.rar", "password")
        .unwrap()
        .list()
        .unwrap();
    assert_eq!(
        entries.next().unwrap().unwrap().filename,
        PathBuf::from(".gitignore")
    );
}

#[test]
fn no_password_list_encrypted_headers() {
    // Password needed in order to list contents
    let mut entries = Archive::new("data/comment-hpw-password.rar")
        .unwrap()
        .list()
        .unwrap();
    let err = entries.next().unwrap().unwrap_err();
    assert_eq!(err.code, Code::MissingPassword);
    assert_eq!(err.when, When::Read);
    assert!(err.data.is_none());
}

#[test]
fn extract_encrypted_headers() {
    let t = TempDir::new("unrar").unwrap();
    Archive::with_password("data/comment-hpw-password.rar", "password")
        .unwrap()
        .extract_to(t.path())
        .unwrap()
        .process()
        .unwrap();
    let mut file = File::open(t.path().join(".gitignore")).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    assert_eq!(s, "target\nCargo.lock\n");
}
