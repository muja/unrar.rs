use native;
use regex::Regex;
use libc::{c_uint, c_long, c_int};
use std::str;
use std::fmt;
use std::ffi::CStr;
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
    ListSplit = native::RAR_OM_LIST_INCSPLIT
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Skip = native::RAR_SKIP,
    Test = native::RAR_TEST,
    Extract = native::RAR_EXTRACT
}

lazy_static! {
    static ref MULTIPART: Regex = Regex::new(r"(\.part)(\d+)(\.rar$)|(\.r?)(\d+)($)").unwrap();
    static ref REGEX: Regex = Regex::new(&[MULTIPART.as_str(), r"\.rar$"].connect("|")).unwrap();
}

pub struct Archive<'a> {
    filename: &'a str,
    password: Option<&'a str>,
    comments: Option<&'a mut Vec<u8>>
}

impl<'a> Archive<'a> {
    pub fn new(file: &'a str) -> Self {
        Archive {
            filename: file,
            password: None,
            comments: None
        }
    }

    pub fn with_password(file: &'a str, password: &'a str) -> Self {
        Archive {
            filename: file,
            password: Some(password),
            comments: None
        }
    }

    pub fn set_comments(&mut self, comments: &'a mut Vec<u8>) {
        self.comments = Some(comments);
    }

    pub fn list(self) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::List, None, Operation::Skip)
    }

    pub fn list_split(self) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::ListSplit, None, Operation::Skip)
    }

    pub fn extract_to(self, path: &str) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::Extract, Some(path), Operation::Extract)
    }

    pub fn test(self) -> UnrarResult<OpenArchive> {
        self.open(OpenMode::Extract, None, Operation::Test)
    }

    pub fn open(self,
        mode: OpenMode, path: Option<&str>, operation: Operation
    ) -> UnrarResult<OpenArchive> {
        OpenArchive::new(self.filename, mode, self.password, path, operation)
    }
}

pub struct OpenArchive {
    handle: native::Handle,
    operation: Operation,
    destination: Option<String>,
    damaged: bool
}

impl OpenArchive {
    pub fn new(
        filename: &str,
        mode: OpenMode,
        password: Option<&str>,
        destination: Option<&str>,
        operation: Operation
    ) -> UnrarResult<Self> {
        let mut data = native::OpenArchiveData::new(
            cstr!(filename).as_ptr() as *const _,
            mode as u32
        );
        let handle = unsafe {
            native::RAROpenArchive(&mut data as *mut _)
        };
        let result = Code::from(data.open_result).unwrap();
        if handle.is_null() {
            Err(UnrarError::from(result, When::Open))
        } else {
            if let Some(pw) = password {
                unsafe {
                    native::RARSetPassword(handle, cstr!(pw).as_ptr() as *const _)
                }
            }
            let dest = destination.map(|path| cstr!(path));
            let archive = OpenArchive {
                handle: handle,
                destination: dest,
                damaged: false,
                operation: operation
            };
            match result {
                Code::Success => Ok(archive),
                _ => Err(UnrarError::new(result, When::Open, archive))
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
            },
            None => Ok(results)
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
                    _ => 1
                }
            },
            _ => 0
        }
    }
}

bitflags! {
    flags EntryFlags: u32 {
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
    pub next_volume: Option<String>
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
        try!(write!(f, "{}", self.filename));
        if self.is_directory() {
            try!(write!(f, "/"))
        }
        if self.is_split() {
            try!(write!(f, " (partial)"))
        }
        Ok(())
    }
}

impl From<native::HeaderData> for Entry {
    fn from(header: native::HeaderData) -> Self {
        Entry {
            filename: str::from_utf8(
                unsafe { CStr::from_ptr(header.filename.as_ptr()) }.to_bytes()
            ).unwrap().into(),
            flags: EntryFlags::from_bits(header.flags).unwrap(),
            unpacked_size: header.unp_size,
            file_crc: header.file_crc,
            file_time: header.file_time,
            method: header.method,
            file_attr: header.file_attr,
            next_volume: None
        }
    }
}

impl Iterator for OpenArchive {
    type Item = UnrarResult<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        // The damaged flag was set, don't attempt to read any further, stop
        if self.damaged {
            return None
        }
        let mut volume = None;
        unsafe {
            native::RARSetCallback(self.handle, Self::callback, &mut volume as *mut _ as c_long)
        }
        let mut header = native::HeaderData::default();
        let read_result = Code::from(unsafe {
            native::RARReadHeader(self.handle, &mut header as *mut _) as u32
        } ).unwrap();
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
                } ).unwrap();
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
                    },
                    _ => {
                        self.damaged = true;
                        Some(Err(UnrarError::from(process_result, When::Process)))
                    }
                }
            },
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

pub fn is_archive(s: &str) -> bool {
    REGEX.find(s).is_some()
}

pub fn is_multipart(s: &str) -> bool {
    MULTIPART.find(s).is_some()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_is_archive() {
        assert_eq!(is_archive("archive.rar"), true);
        assert_eq!(is_archive("archive.part1.rar"), true);
        assert_eq!(is_archive("archive.part100.rar"), true);
        assert_eq!(is_archive("archive.r10"), true);
        assert_eq!(is_archive("archive.part1rar"), false);
        assert_eq!(is_archive("archive.rar\n"), false);
        assert_eq!(is_archive("archive.zip"), false);
    }
}
