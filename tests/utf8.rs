use std::path::PathBuf;
use unrar::Archive;

#[test]
fn unicode_list() {
    let mut entries = Archive::new("data/unicode.rar").open_for_listing().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, PathBuf::from("te…―st✌"));
}

#[test]
fn unicode_file() {
    let mut entries = Archive::new("data/unicodefilename❤️.rar").open_for_listing().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, PathBuf::from(".gitignore"));
}

#[test]
fn unicode_extract_to() {
    let parent = tempfile::tempdir().unwrap();
    let unicode_file = parent.path().join("unicodefilename❤️.txt");
    let archive = Archive::new("data/version.rar").open_for_processing().unwrap();
    let archive = archive.read_header().unwrap().unwrap();
    archive.extract_to(&unicode_file).expect("extraction failed");
    assert_eq!("unrar-0.4.0", std::fs::read_to_string(unicode_file).expect("read failed"));
}

#[test]
fn extract_with_unicode_base() {
    let parent = tempfile::tempdir().unwrap();
    let unicode_dir = parent.path().join("unicodefilename❤️");
    std::fs::create_dir(&unicode_dir).expect("create dir");
    let archive = Archive::new("data/version.rar")
        .open_for_processing()
        .unwrap()
        .read_header()
        .unwrap()
        .unwrap();
    archive.extract_with_base(&unicode_dir).expect("extraction failed");
    assert_eq!(
        "unrar-0.4.0",
        std::fs::read_to_string(unicode_dir.join("VERSION")).expect("read failed")
    );
}

#[test]
fn unicode_entry() {
    let archive = Archive::new("data/unicode-entry.rar").open_for_listing().unwrap();
    let archive = archive.read_header().unwrap().unwrap();
    assert_eq!(archive.entry().filename.as_os_str(), "unicodefilename❤️.txt");
}

#[test]
fn unicode_entry_process_mode() {
    let archive = Archive::new("data/unicode-entry.rar").open_for_processing().unwrap();
    let archive = archive.read_header().unwrap().unwrap();
    assert_eq!(archive.entry().filename.as_os_str(), "unicodefilename❤️.txt");
    assert_eq!(&String::from_utf8(archive.read().unwrap().0).unwrap(), "foobar\n");
}

#[test]
fn unicode_entry_extract() {
    let parent = tempfile::tempdir().unwrap();
    let archive = Archive::new("data/unicode-entry.rar").open_for_processing().unwrap();
    let archive = archive.read_header().unwrap().unwrap();
    archive.extract_with_base(&parent).expect("extract");
    let entries = std::fs::read_dir(&parent).expect("read_dir").collect::<Result<Vec<_>, _>>().expect("read_dir[0]");
    assert_eq!(entries.len(), 1);
    assert_eq!(&entries[0].file_name(), "unicodefilename❤️.txt");
}
