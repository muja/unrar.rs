extern crate unrar;

use unrar::archive::VolumeInfo;
use unrar::Archive;

#[test]
fn volume() {
    let archive = Archive::new("data/version.rar").unwrap().list().unwrap();
    assert_eq!(archive.volume_info(), VolumeInfo::None);

    let archive = Archive::new("data/archive.part1.rar")
        .unwrap()
        .list()
        .unwrap();
    assert_eq!(archive.volume_info(), VolumeInfo::First);

    let archive = Archive::new("data/100M.part00002.rar")
        .unwrap()
        .list()
        .unwrap();
    assert_eq!(archive.volume_info(), VolumeInfo::Subsequent);
}

#[test]
fn locked() {
    let archive = Archive::new("data/locked.rar").unwrap().list().unwrap();
    assert!(archive.is_locked());

    let archive = Archive::new("data/version.rar").unwrap().list().unwrap();
    assert!(!archive.is_locked());
}

#[test]
fn recovery_record() {
    let archive = Archive::new("data/recovery-record.rar")
        .unwrap()
        .list()
        .unwrap();
    assert!(archive.has_recovery_record());

    let archive = Archive::new("data/version.rar").unwrap().list().unwrap();
    assert!(!archive.has_recovery_record());
}

#[test]
fn archive_comment() {
    let archive = Archive::new("data/comment.rar").unwrap().list().unwrap();
    assert!(archive.has_comment());

    let archive = Archive::new("data/version.rar").unwrap().list().unwrap();
    assert!(!archive.has_comment());
}

#[test]
fn encrypted_headers() {
    let archive = Archive::new("data/comment-hpw-password.rar")
        .unwrap()
        .list()
        .unwrap();
    assert!(archive.has_encrypted_headers());

    let archive = Archive::new("data/version.rar").unwrap().list().unwrap();
    assert!(!archive.has_encrypted_headers());
}

#[test]
fn solid() {
    let archive = Archive::new("data/solid.rar").unwrap().list().unwrap();
    assert!(archive.is_solid());

    let archive = Archive::new("data/version.rar").unwrap().list().unwrap();
    assert!(!archive.is_solid());
}
