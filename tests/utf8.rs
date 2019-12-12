extern crate tempdir;
extern crate unrar;

use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn foobar_list() {
    let mut entries = unrar::Archive::new("data/utf8.rar".into()).list().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, "foo—bar");
}
