extern crate unrar;

use std::str;

#[test]
fn version_cat() {
    let bytes = &unrar::Archive::new("data/version.rar".into())
        .read_bytes("VERSION")
        .unwrap();
    let s = str::from_utf8(bytes).unwrap();
    assert_eq!(s, "unrar-0.4.0");
}
