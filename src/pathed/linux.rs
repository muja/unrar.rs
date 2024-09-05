use super::Nulable;
use std::ffi::{CString, CStr};
use std::path::{Path, PathBuf};

pub(crate) type RarString = CString;
pub(crate) type RarStr = CStr;

pub(crate) fn construct<E: crate::RarError>(path: &Path) -> Result<RarString, Nulable<E>> {
    CString::new(path.as_os_str().as_encoded_bytes()).map_err(Nulable::from)
}

pub(crate) fn process_file(
    handle: *const unrar_sys::Handle,
    operation: i32,
    dest_path: Option<&RarStr>,
    dest_name: Option<&RarStr>,
) -> u32 {
    unsafe {
        unrar_sys::RARProcessFile(
            handle,
            operation,
            dest_path.map(|path| path.as_ptr().cast()).unwrap_or(std::ptr::null()),
            dest_name.map(|file| file.as_ptr().cast()).unwrap_or(std::ptr::null()),
        )
    }
}

pub(crate) fn preprocess_extract<E: crate::RarError>(
    base: Option<&Path>,
    filename: &PathBuf,
) -> Result<(Option<RarString>, Option<RarString>), Nulable<E>> {
    construct(&base.unwrap_or(".".as_ref()).join(filename)).map(|e| (None, Some(e)))
}
