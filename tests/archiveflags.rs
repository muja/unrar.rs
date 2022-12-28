extern crate unrar;

use unrar::Archive;
use unrar::VolumeInfo;

#[test]
fn volume() {
    let archive = Archive::new("data/version.rar").open_for_listing().unwrap();
    assert_eq!(archive.volume_info(), VolumeInfo::None);

    let archive = Archive::new("data/archive.part1.rar").open_for_listing().unwrap();
    assert_eq!(archive.volume_info(), VolumeInfo::First);

    let archive = Archive::new("data/100M.part00002.rar").open_for_listing().unwrap();
    assert_eq!(archive.volume_info(), VolumeInfo::Subsequent);
}

#[test]
fn locked() {
    let archive = Archive::new("data/locked.rar").open_for_listing().unwrap();
    assert!(archive.is_locked());

    let archive = Archive::new("data/version.rar").open_for_listing().unwrap();
    assert!(!archive.is_locked());
}

#[test]
fn recovery_record() {
    let archive = Archive::new("data/recovery-record.rar").open_for_listing().unwrap();
    assert!(archive.has_recovery_record());

    let archive = Archive::new("data/version.rar").open_for_listing().unwrap();
    assert!(!archive.has_recovery_record());
}

#[test]
fn archive_comment() {
    let archive = Archive::new("data/comment.rar").open_for_listing().unwrap();
    assert!(archive.has_comment());

    let archive = Archive::new("data/version.rar").open_for_listing().unwrap();
    assert!(!archive.has_comment());
}

#[test]
fn encrypted_headers() {
    let archive = Archive::new("data/comment-hpw-password.rar").open_for_listing().unwrap();
    assert!(archive.has_encrypted_headers());

    let archive = Archive::new("data/version.rar").open_for_listing().unwrap();
    assert!(!archive.has_encrypted_headers());
}

#[test]
fn solid() {
    let archive = Archive::new("data/solid.rar").open_for_listing().unwrap();
    assert!(archive.is_solid());

    let archive = Archive::new("data/version.rar").open_for_listing().unwrap();
    assert!(!archive.is_solid());
}
