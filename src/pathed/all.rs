use super::Nulable;
use std::path::{Path, PathBuf};
use widestring::{WideCString, WideCStr};

pub(crate) type RarString = WideCString;
pub(crate) type RarStr = WideCStr;

pub(crate) fn construct<E: crate::RarError>(path: &Path) -> Result<RarString, Nulable<E>> {
    Ok(WideCString::from_os_str(path)?)
}

pub(crate) fn process_file(
    handle: *const unrar_sys::Handle,
    operation: i32,
    dest_path: Option<&RarStr>,
    dest_name: Option<&RarStr>,
) -> u32 {
    unsafe {
        unrar_sys::RARProcessFileW(
            handle,
            operation,
            dest_path.map(|path| path.as_ptr().cast()).unwrap_or(std::ptr::null()),
            dest_name.map(|file| file.as_ptr().cast()).unwrap_or(std::ptr::null()),
        )
    }
}

pub(crate) fn preprocess_extract<E: crate::RarError>(
    base: Option<&Path>,
    _filename: &PathBuf,
) -> Result<(Option<RarString>, Option<RarString>), Nulable<E>> {
    base.map(construct).transpose().map(|base| (base, None))
}
