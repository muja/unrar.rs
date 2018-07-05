use native;
use regex::Regex;
use libc::{c_uint, c_long, c_int};
use std::str;
use std::fmt;
use std::ffi::CStr;
use std::iter::repeat;
use std::slice;
use error::*;

macro_rules! cstr {
    ($e:expr) => ({
        let mut owned: String = $e.into();
        owned.push_str("\0");
        owned
    })
}

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

macro_rules! mp_ext { () => (r"(\.part|\.r?)(\d+)(\.rar|.{0})$") }
lazy_static! {
    static ref MULTIPART_EXTENSION: Regex = Regex::new(mp_ext!()).unwrap();
    static ref EXTENSION: Regex = Regex::new(concat!(mp_ext!(), r"|\.rar$")).unwrap();
}

pub struct Archive<'a> {
    filename: String,
    password: Option<String>,
    comments: Option<&'a mut Vec<u8>>,
}

pub type Glob = String;

impl<'a> Archive<'a> {
    /// Creates an `Archive` object to operate on a plain RAR archive.
    pub fn new(file: String) -> Self {
        Archive {
            filename: file,
            password: None,
            comments: None,
        }
    }

    /// Creates an `Archive` object to operate on a password encrypted RAR archive.
    pub fn with_password(file: String, password: String) -> Self {
        Archive {
            filename: file,
            password: Some(password),
            comments: None,
        }
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
        MULTIPART_EXTENSION.captures(&self.filename).map(|captures| {
            let mut replacement = String::from(captures.at(1).unwrap());
            replacement.push_str(&repeat("?")
                .take(captures.at(2).unwrap().len())
                .collect::<String>());
            replacement.push_str(captures.at(3).unwrap());
            self.filename.replace(captures.at(0).unwrap(), &replacement)
        })
    }

    /// Returns a glob string covering all parts of the multipart collection or
    /// a copy of the underlying archive's filename if it's a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn all_parts(&self) -> Glob {
        match self.all_parts_option() {
            Some(x) => x,
            None => self.filename.clone(),
        }
    }

    /// Returns the nth part of this multi-part collection or `None`
    /// if the underlying archive is single part
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn nth_part(&self, n: i32) -> Option<String> {
        MULTIPART_EXTENSION.captures(&self.filename).map(|captures| {
            let mut replacement = String::from(captures.at(1).unwrap());
            // `n` padded with zeroes to the length of archive's number's length
            replacement.push_str(&format!("{:01$}", n, captures.at(2).unwrap().len()));
            replacement.push_str(captures.at(3).unwrap());
            self.filename.replace(captures.at(0).unwrap(), &replacement)
        })
    }

    /// Return the first part of the multipart collection or `None`
    /// if the underlying archive is single part
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn first_part_option(&self) -> Option<String> {
        self.nth_part(1)
    }

    /// Returns the first part of the multipart collection or
    /// a copy of the underlying archive's filename if it's a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn first_part(&self) -> String {
        match self.nth_part(1) {
            Some(x) => x,
            None => self.filename.clone(),
        }
    }

    /// Changes the filename to point to the first part of the multipart collection.
    /// Does nothing if it is a single-part archive.
    ///
    /// This method does not make any FS operations and operates purely on strings.
    pub fn as_first_part(&mut self) {
        self.first_part_option().map(|fp| self.filename = fp);
    }

    /// Opens the underlying archive for listing its contents
    pub fn list(self) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::List, None, Operation::Skip)
    }

    /// Opens the underlying archive for listing its contents
    /// without omitting or pooling split entries
    pub fn list_split(self) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::ListSplit, None, Operation::Skip)
    }

    /// Opens the underlying archive for extracting to the given directory.
    pub fn extract_to(self, path: String) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::Extract, Some(path), Operation::Extract)
    }

    /// Opens the underlying archive for testing.
    pub fn test(self) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::Extract, None, Operation::Test)
    }

    /// Opens the underlying archive with the provided parameters.
    pub fn open(self,
                mode: OpenMode,
                path: Option<String>,
                operation: Operation)
                -> UnrarResult<OpenArchive> {
        OpenArchive::new(self.filename, mode, self.password, path, operation)
    }

    /// Returns the bytes for a particular file.
    pub fn read_bytes(self, entry: &str) -> UnrarResult<Vec<u8>> {
        let archive = self.open(OpenMode::Extract, None, Operation::Test).unwrap();
        archive.read_bytes(entry)
    }
}

#[derive(Debug)]
pub struct OpenArchive {
    handle: native::Handle,
    operation: Operation,
    destination: Option<String>,
    damaged: bool,
}

impl OpenArchive {
    fn new(filename: String,
           mode: OpenMode,
           password: Option<String>,
           destination: Option<String>,
           operation: Operation)
           -> UnrarResult<Self> {
        let mut data = native::OpenArchiveData::new(cstr!(filename).as_ptr() as *const _,
                                                    mode as u32);
        let handle = unsafe { native::RAROpenArchive(&mut data as *mut _) };
        let result = Code::from(data.open_result).unwrap();
        if handle.is_null() {
            Err(UnrarError::from(result, When::Open))
        } else {
            if let Some(pw) = password {
                unsafe { native::RARSetPassword(handle, cstr!(pw).as_ptr() as *const _) }
            }
            let dest = destination.map(|path| cstr!(path));
            let archive = OpenArchive {
                handle: handle,
                destination: dest,
                damaged: false,
                operation: operation,
            };
            match result {
                Code::Success => Ok(archive),
                _ => Err(UnrarError::new(result, When::Open, archive)),
            }
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

    extern "C" fn callback(msg: c_uint, user_data: c_long, p1: c_long, p2: c_long) -> c_int {
        // println!("msg: {}, user_data: {}, p1: {}, p2: {}", msg, user_data, p1, p2);
        match msg {
            native::UCM_CHANGEVOLUME => {
                let ptr = p1 as *const _;
                let next = str::from_utf8(unsafe { CStr::from_ptr(ptr) }.to_bytes()).unwrap();
                let our_option = unsafe { &mut *(user_data as *mut Option<String>) };
                *our_option = Some(String::from(next));
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

    extern "C" fn callback_bytes(msg: c_uint, user_data: c_long, p1: c_long, p2: c_long) -> c_int {
        // println!("msg: {}, user_data: {}, p1: {}, p2: {}", msg, user_data, p1, p2);
        match msg {
            native::UCM_PROCESSDATA => {
                let vec = unsafe { &mut *(user_data as *mut Vec<u8>) };
                let bytes = unsafe { slice::from_raw_parts(p1 as *const _, p2 as usize) };
                vec.extend_from_slice(bytes);
                1
            }
            _ => 0,
        }
    }

    pub fn read_bytes(self, entry_filename: &str) -> UnrarResult<Vec<u8>> {
        let mut bytes = Vec::new();
        loop {
            let mut header = native::HeaderData::default();
            let read_result =
                Code::from(unsafe { native::RARReadHeader(self.handle, &mut header as *mut _) as u32 })
                .unwrap();
            match read_result {
                Code::Success => {
                    let mut entry = Entry::from(header);
                    if entry.filename != entry_filename {
                        let process_result = Code::from(unsafe {
                            native::RARProcessFile(
                                self.handle,
                                Operation::Skip as i32,
                                0 as *const _,
                                0 as *const _
                                ) as u32 }).unwrap();
                        match process_result {
                            Code::Success => continue,
                            _ => return Err(UnrarError::from(process_result, When::Process))
                        }
                    }

                    // So we have the right entry, now set the
                    // callback and read it
                    unsafe {
                        native::RARSetCallback(self.handle, Self::callback_bytes, &mut bytes as *mut _ as c_long)
                    }
                    let process_result = Code::from(unsafe {
                        native::RARProcessFile(
                            self.handle,
                            Operation::Test as i32,
                            0 as *const _,
                            0 as *const _
                            ) as u32 }).unwrap();
                    match process_result {
                        Code::Success => break,
                        _ => return Err(UnrarError::from(process_result, When::Process))
                    }
                }
                _ => return Err(UnrarError::from(read_result, When::Read))
            }
        }
        Ok(bytes)
    }
}

impl Iterator for OpenArchive {
    type Item = UnrarResult<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        // The damaged flag was set, don't attempt to read any further, stop
        if self.damaged {
            return None;
        }
        let mut volume = None;
        unsafe {
            native::RARSetCallback(self.handle, Self::callback, &mut volume as *mut _ as c_long)
        }
        let mut header = native::HeaderData::default();
        let read_result =
            Code::from(unsafe { native::RARReadHeader(self.handle, &mut header as *mut _) as u32 })
                .unwrap();
        match read_result {
            Code::Success => {
                let process_result = Code::from(unsafe {
                                         native::RARProcessFile(
                        self.handle,
                        self.operation as i32,
                        self.destination.as_ref().map(
                            |x| x.as_ptr() as *const _
                        ).unwrap_or(0 as *const _),
                        0 as *const _
                    ) as u32
                                     })
                                         .unwrap();
                match process_result {
                    Code::Success | Code::EOpen => {
                        let mut entry = Entry::from(header);
                        // EOpen on Process: Next volume not found
                        if process_result == Code::EOpen {
                            entry.next_volume = volume;
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
            native::RARCloseArchive(self.handle);
        }
    }
}

bitflags! {
    pub flags EntryFlags: u32 {
        const SPLIT_BEFORE = 0x1,
        const SPLIT_AFTER = 0x2,
        const ENCRYPTED = 0x4,
        // const RESERVED = 0x8,
        const SOLID = 0x10,
        const DIRECTORY = 0x20,
    }
}

#[derive(Debug)]
pub struct Entry {
    pub filename: String,
    pub flags: EntryFlags,
    pub unpacked_size: u32,
    pub file_crc: u32,
    pub file_time: u32,
    pub method: u32,
    pub file_attr: u32,
    pub next_volume: Option<String>,
}

impl Entry {
    pub fn is_split(&self) -> bool {
        self.flags.contains(SPLIT_BEFORE) || self.flags.contains(SPLIT_AFTER)
    }

    pub fn is_directory(&self) -> bool {
        self.flags.contains(DIRECTORY)
    }

    pub fn is_encrypted(&self) -> bool {
        self.flags.contains(ENCRYPTED)
    }

    pub fn is_file(&self) -> bool {
        !self.is_directory()
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.filename)?;
        if self.is_directory() {
            write!(f, "/")?
        }
        if self.is_split() {
            write!(f, " (partial)")?
        }
        Ok(())
    }
}

impl From<native::HeaderData> for Entry {
    fn from(header: native::HeaderData) -> Self {
        Entry {
            filename: str::from_utf8(unsafe { CStr::from_ptr(header.filename.as_ptr()) }
                    .to_bytes())
                .unwrap()
                .into(),
            flags: EntryFlags::from_bits(header.flags).unwrap(),
            unpacked_size: header.unp_size,
            file_crc: header.file_crc,
            file_time: header.file_time,
            method: header.method,
            file_attr: header.file_attr,
            next_volume: None,
        }
    }
}

pub fn is_archive(s: &str) -> bool {
    EXTENSION.find(s).is_some()
}

pub fn is_multipart(s: &str) -> bool {
    MULTIPART_EXTENSION.find(s).is_some()
}

#[cfg(test)]
mod tests {
    use super::Archive;
    #[test]
    fn glob() {
        assert_eq!(Archive::new("arc.part0010.rar".into()).all_parts(),
                   "arc.part????.rar");
        assert_eq!(Archive::new("archive.r100".into()).all_parts(),
                   "archive.r???");
        assert_eq!(Archive::new("archive.r9".into()).all_parts(), "archive.r?");
        assert_eq!(Archive::new("archive.999".into()).all_parts(),
                   "archive.???");
        assert_eq!(Archive::new("archive.rar".into()).all_parts(),
                   "archive.rar");
        assert_eq!(Archive::new("random_string".into()).all_parts(),
                   "random_string");
        assert_eq!(Archive::new("v8/v8.rar".into()).all_parts(), "v8/v8.rar");
        assert_eq!(Archive::new("v8/v8".into()).all_parts(), "v8/v8");
    }

    #[test]
    fn first_part() {
        assert_eq!(Archive::new("arc.part0010.rar".into()).first_part(),
                   "arc.part0001.rar");
        assert_eq!(Archive::new("archive.r100".into()).first_part(),
                   "archive.r001");
        assert_eq!(Archive::new("archive.r9".into()).first_part(), "archive.r1");
        assert_eq!(Archive::new("archive.999".into()).first_part(),
                   "archive.001");
        assert_eq!(Archive::new("archive.rar".into()).first_part(),
                   "archive.rar");
        assert_eq!(Archive::new("random_string".into()).first_part(),
                   "random_string");
        assert_eq!(Archive::new("v8/v8.rar".into()).first_part(), "v8/v8.rar");
        assert_eq!(Archive::new("v8/v8".into()).first_part(), "v8/v8");
    }

    #[test]
    fn is_archive() {
        assert_eq!(super::is_archive("archive.rar"), true);
        assert_eq!(super::is_archive("archive.part1.rar"), true);
        assert_eq!(super::is_archive("archive.part100.rar"), true);
        assert_eq!(super::is_archive("archive.r10"), true);
        assert_eq!(super::is_archive("archive.part1rar"), false);
        assert_eq!(super::is_archive("archive.rar\n"), false);
        assert_eq!(super::is_archive("archive.zip"), false);
    }
}
