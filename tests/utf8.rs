use unrar::Archive;
use std::path::PathBuf;

#[test]
fn unicode_list() {
    let mut entries = Archive::new("data/unicode.rar").open_for_listing().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, PathBuf::from("te…―st✌"));
}

#[test]
fn unicode_file() {
    #[cfg(target_os = "linux")]
    {
        // on Linux hosts, we need to set the locale, otherwise the widestring
        // which is passed to the underlying library will not be interpreted correctly if it
        // contains unicode characters
        let locale = std::ffi::CString::new("en_US.utf8").unwrap();
        unsafe { libc::setlocale(libc::LC_ALL, locale.as_ptr()) };
    }
    let mut entries = Archive::new("data/unicodefilename❤️.rar").open_for_listing().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().filename, PathBuf::from(".gitignore"));
}
