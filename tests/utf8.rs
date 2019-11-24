extern crate unrar;

use unrar::Archive;
use std::path::PathBuf;

#[test]
fn unicode_list() {
    let mut entries = Archive::new("data/unicode.rar").list().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, PathBuf::from("te…―st✌"));
}
