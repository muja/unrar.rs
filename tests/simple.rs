use std::path::PathBuf;

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
