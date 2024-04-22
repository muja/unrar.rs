use std::{env::set_current_dir, fs::read_to_string, path::{Path, PathBuf}};

use tempfile::tempdir;

#[test]
fn version_list() {
    let mut archive = unrar::Archive::new("data/version.rar")
        .open_for_listing()
        .unwrap();
    assert_eq!(
        archive.next().unwrap().unwrap().filename,
        PathBuf::from("VERSION")
    );
}

#[test]
fn version_extract_to_absolute_path() {
    let archive = unrar::Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap();

    let tmp_dir = tempdir().unwrap();

    let header = archive.read_header().unwrap().unwrap();
    let filename = header.entry().filename.clone();
    header.extract_to(tmp_dir.path().join(filename)).unwrap();

    assert_eq!(read_to_string(tmp_dir.path().join("VERSION")).unwrap(), "unrar-0.4.0");
}

#[test]
fn version_extract_to_relative_path() {
    let archive = unrar::Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap();

    let tmp_dir = tempdir().unwrap();
    set_current_dir(&tmp_dir.path()).unwrap();

    let header = archive.read_header().unwrap().unwrap();
    header.extract_to("some-folder/VERSION").unwrap();

    assert_eq!(read_to_string(tmp_dir.path().join("some-folder/VERSION")).unwrap(), "unrar-0.4.0");
}

#[test]
fn version_cat() {
    let bytes = unrar::Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap()
        .read_header()
        .unwrap()
        .unwrap()
        .read()
        .unwrap()
        .0;
    let s = String::from_utf8(bytes).unwrap();
    assert_eq!(s, "unrar-0.4.0");
}
