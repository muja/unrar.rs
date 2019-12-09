extern crate unrar;

use unrar::Archive;

#[test]
fn unicode_list() {
    let mut entries = Archive::new("data/unicode.rar".into()).list().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, "te…―st✌");
}
