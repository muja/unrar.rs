use error::*;
use native;
use regex::Regex;
use std::borrow::Cow;
use std::ffi::CString;
use std::fmt;
use std::iter::repeat;
use std::os::raw::{c_int, c_uint};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::str;
use widestring::WideCString;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    List = native::RAR_OM_LIST,
    Extract = native::RAR_OM_EXTRACT,
    ListSplit = native::RAR_OM_LIST_INCSPLIT,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Skip = native::RAR_SKIP,
    Test = native::RAR_TEST,
    Extract = native::RAR_EXTRACT,
}

macro_rules! mp_ext {
    () => {
        r"(\.part|\.r?)(\d+)((?:\.rar)?)$"
    };
}
lazy_static! {
    static ref MULTIPART_EXTENSION: Regex = Regex::new(mp_ext!()).unwrap();
    static ref EXTENSION: Regex = Regex::new(concat!(mp_ext!(), r"|\.rar$")).unwrap();
}

pub struct Archive<'a> {
    filename: Cow<'a, Path>,
    password: Option<CString>,
    comments: Option<&'a mut Vec<u8>>,
}

pub type Glob = PathBuf;

impl<'a> Archive<'a> {
    /// Creates an `Archive` object to operate on a plain RAR archive.
    pub fn new<T>(file: &'a T) -> Result<Self, NulError>
    where
        T: AsRef<Path> + ?Sized,
    {
        let _ = WideCString::from_os_str(file.as_ref())?;
        Ok(Archive {
            filename: Cow::Borrowed(file.as_ref()),
            password: None,
            comments: None,
        })
    }

    /// Creates an `Archive` object to operate on a password encrypted RAR archive.
    pub fn with_password<T, U>(file: &'a T, password: &'a U) -> Result<Self, NulError>
    where
        T: AsRef<Path> + ?Sized,
        U: AsRef<str> + ?Sized,
    {
        let _ = WideCString::from_os_str(file.as_ref())?;
        Ok(Archive {
            filename: Cow::Borrowed(file.as_ref()),
            password: Some(CString::new(password.as_ref())?),
            comments: None,
        })
    }

    /// Set the comment buffer of the underlying archive.
    /// Note: Comments are not supported yet so this method will have no effect.
    pub fn set_comments(&mut self, comments: &'a mut Vec<u8>) {
        self.comments = Some(comments);
    }

    /// Returns `true` if the filename matches a RAR archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn is_archive(&self) -> bool {
        is_archive(&self.filename)
    }

    /// Returns `true` if the filename matches a part of a multipart collection, `false` otherwise
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn is_multipart(&self) -> bool {
        is_multipart(&self.filename)
    }

    /// Returns a glob string covering all parts of the multipart collection or `None`
    /// if the underlying archive is a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn all_parts_option(&self) -> Option<Glob> {
        get_rar_extension(&self.filename)
            .and_then(|full_ext| {
                MULTIPART_EXTENSION.captures(&full_ext).map(|captures| {
                    let mut replacement = String::from(captures.get(1).unwrap().as_str());
                    replacement.push_str(
                        &repeat("?")
                            .take(captures.get(2).unwrap().as_str().len())
                            .collect::<String>(),
                    );
                    replacement.push_str(captures.get(3).unwrap().as_str());
                    full_ext.replace(captures.get(0).unwrap().as_str(), &replacement)
                })
            })
            .and_then(|new_ext| {
                self.filename
                    .file_stem()
                    .map(|x| Path::new(x).with_extension(&new_ext[1..]))
            })
    }

    /// Returns a glob string covering all parts of the multipart collection or
    /// a copy of the underlying archive's filename if it's a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn all_parts(&self) -> Glob {
        match self.all_parts_option() {
            Some(x) => x,
            None => self.filename.to_path_buf(),
        }
    }

    /// Returns the nth part of this multi-part collection or `None`
    /// if the underlying archive is single part
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn nth_part(&self, n: i32) -> Option<PathBuf> {
        get_rar_extension(&self.filename)
            .and_then(|full_ext| {
                MULTIPART_EXTENSION.captures(&full_ext).map(|captures| {
                    let mut replacement = String::from(captures.get(1).unwrap().as_str());
                    // `n` padded with zeroes to the length of archive's number's length
                    replacement.push_str(&format!(
                        "{:01$}",
                        n,
                        captures.get(2).unwrap().as_str().len()
                    ));
                    replacement.push_str(captures.get(3).unwrap().as_str());
                    full_ext.replace(captures.get(0).unwrap().as_str(), &replacement)
                })
            })
            .and_then(|new_ext| {
                self.filename
                    .file_stem()
                    .map(|x| Path::new(x).with_extension(&new_ext[1..]))
            })
    }

    /// Return the first part of the multipart collection or `None`
    /// if the underlying archive is single part
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn first_part_option(&self) -> Option<PathBuf> {
        self.nth_part(1)
    }

    /// Returns the first part of the multipart collection or
    /// a copy of the underlying archive's filename if it's a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn first_part(&self) -> PathBuf {
        match self.nth_part(1) {
            Some(x) => x,
            None => self.filename.to_path_buf(),
        }
    }

    /// Changes the filename to point to the first part of the multipart collection.
    /// Does nothing if it is a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn as_first_part(&mut self) {
        self.first_part_option()
            .map(|fp| self.filename = Cow::Owned(fp));
    }

    /// Opens the underlying archive for listing its contents
    pub fn list(self) -> UnrarResult<OpenArchive> {
        self.open::<&str>(OpenMode::List, None, Operation::Skip)
    }

    /// Opens the underlying archive for listing its contents
    /// without omitting or pooling split entries
    pub fn list_split(self) -> UnrarResult<OpenArchive> {
        self.open::<&str>(OpenMode::ListSplit, None, Operation::Skip)
    }

    /// Opens the underlying archive for extracting to the given directory.
    ///
    /// # Panics
    ///
    /// Panics if `path` contains nul values.
    pub fn extract_to<T: AsRef<Path>>(self, path: T) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::Extract, Some(path), Operation::Extract)
    }

    /// Opens the underlying archive for testing.
    pub fn test(self) -> UnrarResult<OpenArchive> {
        self.open::<&str>(OpenMode::Extract, None, Operation::Test)
    }

    /// Opens the underlying archive with the provided parameters.
    ///
    /// # Panics
    ///
    /// Panics if `path` contains nul values.
    pub fn open<T: AsRef<Path>>(
        self,
        mode: OpenMode,
        path: Option<T>,
        operation: Operation,
    ) -> UnrarResult<OpenArchive> {
        OpenArchive::new(
            &self.filename,
            mode,
            self.password,
            path.as_ref().map(|x| x.as_ref()),
            operation,
        )
    }
}

bitflags! {
    #[derive(Default)]
    struct ArchiveFlags: u32 {
        const VOLUME = native::ROADF_VOLUME;
        const COMMENT = native::ROADF_COMMENT;
        const LOCK = native::ROADF_LOCK;
        const SOLID = native::ROADF_SOLID;
        const NEW_NUMBERING = native::ROADF_NEWNUMBERING;
        const SIGNED = native::ROADF_SIGNED;
        const RECOVERY = native::ROADF_RECOVERY;
        const ENC_HEADERS = native::ROADF_ENCHEADERS;
        const FIRST_VOLUME = native::ROADF_FIRSTVOLUME;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeInfo {
    None,
    First,
    Subsequent,
}

#[derive(Debug)]
pub struct OpenArchive {
    handle: NonNull<native::HANDLE>,
    operation: Operation,
    destination: Option<WideCString>,
    damaged: bool,
    flags: ArchiveFlags,
}

impl OpenArchive {
    fn new(
        filename: &Path,
        mode: OpenMode,
        password: Option<CString>,
        destination: Option<&Path>,
        operation: Operation,
    ) -> UnrarResult<Self> {
        let destination = match destination {
            Some(dest) => {
                Some(WideCString::from_os_str(&dest).expect("Unexpected nul in destination"))
            }
            None => None,
        };
        // Panic here is our fault. Either something in Archive has added a nul to filename,
        // or filename was not checked for nuls on Archive creation.
        let filename = WideCString::from_os_str(&filename).expect("Unexpected nul in filename");

        let mut data = native::OpenArchiveDataEx::new(filename.as_ptr() as *const _, mode as u32);
        let handle =
            NonNull::new(unsafe { native::RAROpenArchiveEx(&mut data as *mut _) } as *mut _);
        let result = Code::from(data.open_result).unwrap();

        if let Some(handle) = handle {
            if let Some(pw) = password {
                unsafe { native::RARSetPassword(handle.as_ptr(), pw.as_ptr() as *const _) }
            }

            let archive = OpenArchive {
                handle: handle,
                destination: destination,
                damaged: false,
                flags: ArchiveFlags::from_bits(data.flags).unwrap(),
                operation: operation,
            };

            match result {
                Code::Success => Ok(archive),
                _ => Err(UnrarError::new(result, When::Open, archive)),
            }
        } else {
            Err(UnrarError::from(result, When::Open))
        }
    }

    pub fn is_locked(&self) -> bool {
        self.flags.contains(ArchiveFlags::LOCK)
    }

    pub fn has_encrypted_headers(&self) -> bool {
        self.flags.contains(ArchiveFlags::ENC_HEADERS)
    }

    pub fn has_recovery_record(&self) -> bool {
        self.flags.contains(ArchiveFlags::RECOVERY)
    }

    pub fn has_comment(&self) -> bool {
        self.flags.contains(ArchiveFlags::COMMENT)
    }

    /// Solid archive, all files in a single compressed block.
    pub fn is_solid(&self) -> bool {
        self.flags.contains(ArchiveFlags::SOLID)
    }

    /// Indicates whether the archive file is split into multiple volumes or not,
    /// and if so, whether the file is the first volume or not.
    pub fn volume_info(&self) -> VolumeInfo {
        if self.flags.contains(ArchiveFlags::FIRST_VOLUME) {
            VolumeInfo::First
        } else if self.flags.contains(ArchiveFlags::VOLUME) {
            VolumeInfo::Subsequent
        } else {
            VolumeInfo::None
        }
    }

    pub fn process(&mut self) -> UnrarResult<Vec<Entry>> {
        let (ts, es): (Vec<_>, Vec<_>) = self.partition(|x| x.is_ok());
        let mut results: Vec<_> = ts.into_iter().map(|x| x.unwrap()).collect();
        match es.into_iter().map(|x| x.unwrap_err()).next() {
            Some(error) => {
                error.data.map(|x| results.push(x));
                Err(UnrarError::new(error.code, error.when, results))
            }
            None => Ok(results),
        }
    }

    extern "C" fn callback(
        msg: native::UINT,
        user_data: native::LPARAM,
        p1: native::LPARAM,
        p2: native::LPARAM,
    ) -> c_int {
        // println!("msg: {}, user_data: {}, p1: {}, p2: {}", msg, user_data, p1, p2);
        match msg {
            native::UCM_CHANGEVOLUMEW => {
                let ptr = p1 as *const _;
                // 2048 seems to be the buffer size in unrar,
                // also it's the maximum path length since 5.00.
                let next = unsafe { WideCString::from_ptr_with_nul(ptr, 2048) }.ok();
                let our_option = unsafe { &mut *(user_data as *mut Option<WideCString>) };
                *our_option = next;
                match p2 {
                    // Next volume not found. -1 means stop
                    native::RAR_VOL_ASK => -1,
                    // Next volume found, 1 means continue
                    _ => 1,
                }
            }
            _ => 0,
        }
    }
}

impl Iterator for OpenArchive {
    type Item = UnrarResult<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        // The damaged flag was set, don't attempt to read any further, stop
        if self.damaged {
            return None;
        }
        let mut volume: Option<WideCString> = None;
        unsafe {
            native::RARSetCallback(
                self.handle.as_ptr(),
                Self::callback,
                &mut volume as *mut _ as native::LPARAM,
            )
        }
        let mut header = native::HeaderDataEx::default();
        let read_result = Code::from(unsafe {
            native::RARReadHeaderEx(self.handle.as_ptr(), &mut header as *mut _) as u32
        })
        .unwrap();
        match read_result {
            Code::Success => {
                let process_result = Code::from(unsafe {
                    native::RARProcessFileW(
                        self.handle.as_ptr(),
                        self.operation as i32,
                        self.destination
                            .as_ref()
                            .map(|x| x.as_ptr() as *const _)
                            .unwrap_or(std::ptr::null()),
                        std::ptr::null(),
                    ) as u32
                })
                .unwrap();

                match process_result {
                    Code::Success | Code::EOpen => {
                        let mut entry = Entry::from(header);
                        // EOpen on Process: Next volume not found
                        if process_result == Code::EOpen {
                            entry.next_volume = volume.map(|x| PathBuf::from(x.to_os_string()));
                            self.damaged = true;
                            Some(Err(UnrarError::new(process_result, When::Process, entry)))
                        } else {
                            Some(Ok(entry))
                        }
                    }
                    _ => {
                        self.damaged = true;
                        Some(Err(UnrarError::from(process_result, When::Process)))
                    }
                }
            }
            Code::EndArchive => None,
            _ => {
                self.damaged = true;
                Some(Err(UnrarError::from(read_result, When::Read)))
            }
        }
    }
}

impl Drop for OpenArchive {
    fn drop(&mut self) {
        unsafe {
            native::RARCloseArchive(self.handle.as_ptr());
        }
    }
}

fn unpack_unp_size(unp_size: c_uint, unp_size_high: c_uint) -> usize {
    ((unp_size_high as usize) << (8 * std::mem::size_of::<c_uint>())) | (unp_size as usize)
}

bitflags! {
    pub struct EntryFlags: u32 {
        const SPLIT_BEFORE = 0x1;
        const SPLIT_AFTER = 0x2;
        const ENCRYPTED = 0x4;
        // const RESERVED = 0x8;
        const SOLID = 0x10;
        const DIRECTORY = 0x20;
    }
}

#[derive(Debug)]
pub struct Entry {
    pub filename: PathBuf,
    pub flags: EntryFlags,
    pub unpacked_size: usize,
    pub file_crc: u32,
    pub file_time: u32,
    pub method: u32,
    pub file_attr: u32,
    pub next_volume: Option<PathBuf>,
}

impl Entry {
    pub fn is_split(&self) -> bool {
        self.flags.contains(EntryFlags::SPLIT_BEFORE)
            || self.flags.contains(EntryFlags::SPLIT_AFTER)
    }

    pub fn is_directory(&self) -> bool {
        self.flags.contains(EntryFlags::DIRECTORY)
    }

    pub fn is_encrypted(&self) -> bool {
        self.flags.contains(EntryFlags::ENCRYPTED)
    }

    pub fn is_file(&self) -> bool {
        !self.is_directory()
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.filename)?;
        if self.is_directory() {
            write!(f, "/")?
        }
        if self.is_split() {
            write!(f, " (partial)")?
        }
        Ok(())
    }
}

impl From<native::HeaderDataEx> for Entry {
    fn from(header: native::HeaderDataEx) -> Self {
        let filename =
            unsafe { WideCString::from_ptr_with_nul(header.filename_w.as_ptr() as *const _, 1024) }
                .unwrap();

        Entry {
            filename: PathBuf::from(filename.to_os_string()),
            flags: EntryFlags::from_bits(header.flags).unwrap(),
            unpacked_size: unpack_unp_size(header.unp_size, header.unp_size_high),
            file_crc: header.file_crc,
            file_time: header.file_time,
            method: header.method,
            file_attr: header.file_attr,
            next_volume: None,
        }
    }
}

fn get_rar_extension<T: AsRef<Path>>(path: T) -> Option<String> {
    path.as_ref().extension().map(|ext| {
        let pre_ext = path
            .as_ref()
            .file_stem()
            .and_then(|x| Path::new(x).extension());
        match pre_ext {
            Some(pre_ext) => format!(".{}.{}", pre_ext.to_string_lossy(), ext.to_string_lossy()),
            None => format!(".{}", ext.to_string_lossy()),
        }
    })
}

pub fn is_archive(s: &Path) -> bool {
    get_rar_extension(s)
        .and_then(|full_ext| EXTENSION.find(&full_ext).map(|_| ()))
        .is_some()
}

pub fn is_multipart(s: &Path) -> bool {
    get_rar_extension(s)
        .and_then(|full_ext| MULTIPART_EXTENSION.find(&full_ext).map(|_| ()))
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::Archive;
    use std::path::PathBuf;

    #[test]
    fn glob() {
        assert_eq!(
            Archive::new("arc.part0010.rar").unwrap().all_parts(),
            PathBuf::from("arc.part????.rar")
        );
        assert_eq!(
            Archive::new("archive.r100").unwrap().all_parts(),
            PathBuf::from("archive.r???")
        );
        assert_eq!(
            Archive::new("archive.r9").unwrap().all_parts(),
            PathBuf::from("archive.r?")
        );
        assert_eq!(
            Archive::new("archive.999").unwrap().all_parts(),
            PathBuf::from("archive.???")
        );
        assert_eq!(
            Archive::new("archive.rar").unwrap().all_parts(),
            PathBuf::from("archive.rar")
        );
        assert_eq!(
            Archive::new("random_string").unwrap().all_parts(),
            PathBuf::from("random_string")
        );
        assert_eq!(
            Archive::new("v8/v8.rar").unwrap().all_parts(),
            PathBuf::from("v8/v8.rar")
        );
        assert_eq!(
            Archive::new("v8/v8").unwrap().all_parts(),
            PathBuf::from("v8/v8")
        );
    }

    #[test]
    fn first_part() {
        assert_eq!(
            Archive::new("arc.part0010.rar").unwrap().first_part(),
            PathBuf::from("arc.part0001.rar")
        );
        assert_eq!(
            Archive::new("archive.r100").unwrap().first_part(),
            PathBuf::from("archive.r001")
        );
        assert_eq!(
            Archive::new("archive.r9").unwrap().first_part(),
            PathBuf::from("archive.r1")
        );
        assert_eq!(
            Archive::new("archive.999").unwrap().first_part(),
            PathBuf::from("archive.001")
        );
        assert_eq!(
            Archive::new("archive.rar").unwrap().first_part(),
            PathBuf::from("archive.rar")
        );
        assert_eq!(
            Archive::new("random_string").unwrap().first_part(),
            PathBuf::from("random_string")
        );
        assert_eq!(
            Archive::new("v8/v8.rar").unwrap().first_part(),
            PathBuf::from("v8/v8.rar")
        );
        assert_eq!(
            Archive::new("v8/v8").unwrap().first_part(),
            PathBuf::from("v8/v8")
        );
    }

    #[test]
    fn is_archive() {
        assert_eq!(super::is_archive(&PathBuf::from("archive.rar")), true);
        assert_eq!(super::is_archive(&PathBuf::from("archive.part1.rar")), true);
        assert_eq!(
            super::is_archive(&PathBuf::from("archive.part100.rar")),
            true
        );
        assert_eq!(super::is_archive(&PathBuf::from("archive.r10")), true);
        assert_eq!(super::is_archive(&PathBuf::from("archive.part1rar")), false);
        assert_eq!(super::is_archive(&PathBuf::from("archive.rar\n")), false);
        assert_eq!(super::is_archive(&PathBuf::from("archive.zip")), false);
    }

    #[test]
    fn nul_in_input() {
        assert!(Archive::new("\0archive.rar").is_err());
        assert!(Archive::with_password("archive.rar", "un\0rar").is_err());
    }

    #[test]
    #[should_panic(expected = "Unexpected nul in destination")]
    fn nul_in_destination() {
        let _ = Archive::new("archive.rar").unwrap().extract_to("tmp/\0");
    }

    #[test]
    fn combine_size() {
        use super::unpack_unp_size;
        let (high, low) = (1u32, 1464303715u32);
        assert_eq!(unpack_unp_size(low, high), 5759271011);
    }
}
