use std::path::PathBuf;
use unrar::error::RarError;
use unrar::Archive;

#[test]
fn list() {
    // No password needed in order to list contents
    let mut entries = Archive::new("data/crypted.rar").open_for_listing().unwrap();
    assert_eq!(
        entries.next().unwrap().unwrap().filename,
        PathBuf::from(".gitignore")
    );
}

#[test]
fn no_password() {
    let arc = Archive::new("data/crypted.rar")
        .open_for_processing()
        .unwrap();
    let header = arc.read_header();
    assert!(matches!(header, Ok(Some(_))));
    let read_result = header.unwrap().unwrap().read();
    assert!(matches!(read_result, Err(_)));
    let err = read_result.unwrap_err();
    assert_eq!(err, RarError::MissingPassword);
}

#[test]
fn version_cat() {
    let file = Archive::with_password("data/crypted.rar", "unrar")
        .open_for_processing()
        .unwrap()
        .read_header()
        .unwrap()
        .unwrap()
        .read()
        .unwrap()
        .0;
    let s = String::from_utf8(file).unwrap();
    assert_eq!(s, "target\nCargo.lock\n");
}

#[test]
fn list_encrypted_headers() {
    let mut entries = Archive::with_password("data/comment-hpw-password.rar", "password")
        .open_for_listing()
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
        .open_for_listing()
        .unwrap();
    let err = entries.next().unwrap().unwrap_err();
    assert_eq!(err, RarError::MissingPassword);
}

#[test]
fn extract_encrypted_headers() {
    let bytes = Archive::with_password("data/comment-hpw-password.rar", "password")
        .open_for_processing()
        .unwrap()
        .read_header()
        .unwrap()
        .unwrap()
        .read()
        .unwrap()
        .0;
    let s = String::from_utf8(bytes).unwrap();
    assert_eq!(s, "target\nCargo.lock\n");
}
