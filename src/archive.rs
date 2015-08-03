use native;
use regex::Regex;
use libc::{c_uint, c_long, c_int};
use std::str;
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
    damaged: bool,
    error: Option<UnrarError>
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
        match result {
            Code::Success => {
                if let Some(pw) = password {
                    unsafe {
                        native::RARSetPassword(handle, cstr!(pw).as_ptr() as *const _)
                    }
                }
                let dest = destination.map(|path| cstr!(path));
                Ok(OpenArchive {
                    handle: handle,
                    destination: dest,
                    damaged: false,
                    error: None,
                    operation: operation
                })
            },
            _ => Err(UnrarError::from(result, When::Open))
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

#[derive(Debug)]
pub struct Entry {
    pub filename: String,
    pub flags: u32,
    pub unpacked_size: u32,
    pub file_crc: u32,
    pub file_time: u32,
    pub method: u32,
    pub file_attr: u32
}

impl From<native::HeaderData> for Entry {
    fn from(header: native::HeaderData) -> Self {
        Entry {
            filename: str::from_utf8(
                unsafe { CStr::from_ptr(header.filename.as_ptr()) }.to_bytes()
            ).unwrap().into(),
            flags: header.flags,
            unpacked_size: header.unp_size,
            file_crc: header.file_crc,
            file_time: header.file_time,
            method: header.method,
            file_attr: header.file_attr
        }
    }
}

impl Iterator for OpenArchive {
    type Item = UnrarResult<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        // The damaged flag was set, don't attempt to read any further, stop
        if self.damaged {
            // If there is an error stored, return that first, and then return None
            if self.error.is_some() {
                return Some(Err(self.error.take().unwrap()))
            } else {
                return None
            }
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
                        let entry = Entry::from(header);
                        // EOpen on Process: Next volume not found
                        // =======================================
                        // We return the information first,
                        // and set the error flag to return Err on the next `next` call
                        // and after that, return None.
                        // Like this:
                        // next() => Some(Entry("MyFile")) // with flags set correctly etc.
                        // next() => Some(Err(Code::EOpen, When::Process, "next_volume.partXX.rar"))
                        // next() => None
                        if process_result == Code::EOpen {
                            self.damaged = true;
                            self.error = Some(UnrarError::new(
                                process_result, When::Process, volume.unwrap()
                            ));
                        }
                        Some(Ok(entry))
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

impl OpenArchive {
    pub fn process(&mut self) -> UnrarResult<Vec<Entry>> {
        self.collect()
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
