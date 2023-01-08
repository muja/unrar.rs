use super::error::*;
use super::*;
use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_int, c_uint};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use widestring::WideCString;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Skip = native::RAR_SKIP,
    Test = native::RAR_TEST,
    Extract = native::RAR_EXTRACT,
}

bitflags::bitflags! {
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
struct Handle(NonNull<native::Handle>);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { native::RARCloseArchive(self.0.as_ptr() as *const _) };
    }
}

#[derive(Debug)]
pub struct OpenArchive<M: OpenMode, C: Cursor> {
    handle: Handle,
    flags: ArchiveFlags,
    extra: C,
    marker: std::marker::PhantomData<M>,
}
type Userdata<T> = (T, Option<WideCString>);
#[derive(Debug)]
pub struct CursorBeforeHeader;
#[derive(Debug)]
pub struct CursorBeforeFile {
    header: FileHeader,
}
pub trait Cursor {}
impl Cursor for CursorBeforeHeader {}
impl Cursor for CursorBeforeFile {}

#[derive(Debug)]
pub struct Process;
#[derive(Debug)]
pub struct List;
#[derive(Debug)]
pub struct ListSplit;

pub trait OpenMode {
    const CODE: u32;
}
impl OpenMode for Process {
    const CODE: u32 = native::RAR_OM_EXTRACT;
}
impl OpenMode for List {
    const CODE: u32 = native::RAR_OM_LIST;
}
impl OpenMode for ListSplit {
    const CODE: u32 = native::RAR_OM_LIST_INCSPLIT;
}

impl<Mode: OpenMode, C: Cursor> OpenArchive<Mode, C> {
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
}

impl<Mode: OpenMode> OpenArchive<Mode, CursorBeforeHeader> {
    pub(crate) fn new(
        filename: &Path,
        password: Option<&[u8]>,
        recover: Option<&mut Option<Self>>,
    ) -> UnrarResult<Self> {
        let filename = WideCString::from_os_str(&filename).unwrap();

        let mut data = native::OpenArchiveDataEx::new(filename.as_ptr() as *const _, Mode::CODE);
        let handle =
            NonNull::new(unsafe { native::RAROpenArchiveEx(&mut data as *mut _) } as *mut _);

        let arc = handle.and_then(|handle| {
            if let Some(pw) = password {
                let cpw = CString::new(pw).unwrap();
                unsafe { native::RARSetPassword(handle.as_ptr(), cpw.as_ptr() as *const _) }
            }
            Some(OpenArchive {
                handle: Handle(handle),
                flags: ArchiveFlags::from_bits(data.flags).unwrap(),
                extra: CursorBeforeHeader,
                marker: std::marker::PhantomData,
            })
        });
        let result = Code::from(data.open_result as i32).unwrap();

        match (arc, result) {
            (Some(arc), Code::Success) => Ok(arc),
            (arc, _) => {
                recover.and_then(|recover| arc.and_then(|arc| recover.replace(arc)));
                Err(UnrarError::from(result, When::Open))
            }
        }
    }

    pub fn read_header(self) -> Option<UnrarResult<OpenArchive<Mode, CursorBeforeFile>>> {
        Some(read_header(&self.handle)?.map(|entry| OpenArchive {
            extra: CursorBeforeFile { header: entry },
            handle: self.handle,
            flags: self.flags,
            marker: std::marker::PhantomData,
        }))
    }
}

impl Iterator for OpenArchive<List, CursorBeforeHeader> {
    type Item = Result<FileHeader, UnrarError>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_header(&self.handle) {
            Some(Ok(header)) => {
                match Internal::<Skip>::process_file_raw(&self.handle, None, None) {
                    Ok(_) => Some(Ok(header)),
                    Err(s) => Some(Err(s)),
                }
            }
            None => None,
            Some(Err(x)) => Some(Err(x)),
        }
    }
}

impl Iterator for OpenArchive<ListSplit, CursorBeforeHeader> {
    type Item = Result<FileHeader, UnrarError>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_header(&self.handle) {
            Some(Ok(header)) => {
                match Internal::<Skip>::process_file_raw(&self.handle, None, None) {
                    Ok(_) => Some(Ok(header)),
                    Err(s) => Some(Err(s)),
                }
            }
            None => None,
            Some(Err(x)) => Some(Err(x)),
        }
    }
}

impl<M: OpenMode> OpenArchive<M, CursorBeforeFile> {
    // TODO better name
    pub fn entry(&self) -> &FileHeader {
        &self.extra.header
    }

    pub fn skip(self) -> UnrarResult<OpenArchive<M, CursorBeforeHeader>> {
        self.process_file::<Skip>(None, None)
    }

    fn process_file<PM: ProcessMode>(
        self,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> UnrarResult<OpenArchive<M, CursorBeforeHeader>> {
        Ok(self.process_file_x::<PM>(path, file)?.1)
    }

    fn process_file_x<PM: ProcessMode>(
        self,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> UnrarResult<(PM::Output, OpenArchive<M, CursorBeforeHeader>)> {
        let result = Ok((
            Internal::<PM>::process_file_raw(&self.handle, path, file)?,
            OpenArchive {
                extra: CursorBeforeHeader,
                handle: self.handle,
                flags: self.flags,
                marker: std::marker::PhantomData,
            },
        ));
        result
    }
}

impl OpenArchive<Process, CursorBeforeFile> {
    /// Reads the underlying file into a Vec<u8>
    /// Returns the data as well as the owned Archive that can be processed further.
    pub fn read(self) -> UnrarResult<(Vec<u8>, OpenArchive<Process, CursorBeforeHeader>)> {
        Ok(self.process_file_x::<ReadToVec>(None, None)?)
    }

    /// Extracts the file into the current working directory
    /// Returns the OpenArchive for further processing
    pub fn extract(self) -> UnrarResult<OpenArchive<Process, CursorBeforeHeader>> {
        self.process_file::<Extract>(None, None)
    }

    /// Extracts the file into the current working directory
    /// Returns the OpenArchive for further processing
    ///
    /// # Panics
    ///
    /// This function will panic if `base` contains nul characters.
    pub fn extract_with_base<P: AsRef<Path>>(
        self,
        base: P,
    ) -> UnrarResult<OpenArchive<Process, CursorBeforeHeader>> {
        let wdest = WideCString::from_os_str(base.as_ref()).expect("Unexpected nul in destination");
        self.process_file::<Extract>(Some(&wdest), None)
    }

    pub fn extract_to<P: AsRef<Path>>(
        self,
        dest: P,
    ) -> UnrarResult<OpenArchive<Process, CursorBeforeHeader>> {
        let wdest = WideCString::from_os_str(dest.as_ref()).expect("Unexpected nul in destination");
        self.process_file::<Extract>(None, Some(&wdest))
    }
}

fn read_header(handle: &Handle) -> Option<UnrarResult<FileHeader>> {
    unsafe {
        native::RARSetCallback(
            handle.0.as_ptr(),
            Some(Internal::<Skip>::callback),
            std::ptr::null_mut() as *mut u8 as native::LPARAM,
        );
    }
    let mut header = native::HeaderDataEx::default();
    let read_result =
        Code::from(unsafe { native::RARReadHeaderEx(handle.0.as_ptr(), &mut header as *mut _) })
            .unwrap();
    match read_result {
        Code::Success => Some(Ok(header.into())),
        Code::EndArchive => None,
        _ => Some(Err(UnrarError::from(read_result, When::Read))),
    }
}

#[derive(Debug)]
pub struct Skip;
#[derive(Debug)]
pub struct ReadToVec;
#[derive(Debug)]
pub struct Extract;

pub trait ProcessMode: core::fmt::Debug {
    const OPERATION: Operation;
    type Output: core::fmt::Debug + std::default::Default;

    fn process_data(data: &mut Self::Output, other: &[u8]);
}
impl ProcessMode for Skip {
    const OPERATION: Operation = Operation::Skip;
    type Output = ();

    fn process_data(_: &mut Self::Output, _: &[u8]) {}
}
impl ProcessMode for ReadToVec {
    const OPERATION: Operation = Operation::Test;
    type Output = Vec<u8>;

    fn process_data(my: &mut Self::Output, other: &[u8]) {
        my.extend_from_slice(other);
    }
}
impl ProcessMode for Extract {
    const OPERATION: Operation = Operation::Extract;
    type Output = ();

    fn process_data(_: &mut Self::Output, _: &[u8]) {}
}

struct Internal<M: ProcessMode> {
    marker: std::marker::PhantomData<M>,
}

impl<M: ProcessMode> Internal<M> {
    extern "C" fn callback(
        msg: native::UINT,
        user_data: native::LPARAM,
        p1: native::LPARAM,
        p2: native::LPARAM,
    ) -> c_int {
        println!(
            "msg: {}, user_data: {}, p1: {}, p2: {}",
            msg, user_data, p1, p2
        );
        if user_data == 0 {
            return 0;
        }
        let user_data = unsafe { &mut *(user_data as *mut Userdata<M::Output>) };
        match msg {
            native::UCM_CHANGEVOLUMEW => {
                // 2048 seems to be the buffer size in unrar,
                // also it's the maximum path length since 5.00.
                let next = unsafe { WideCString::from_ptr_truncate(p1 as *const _, 2048) };
                user_data.1 = Some(next);
                match p2 {
                    // Next volume not found. -1 means stop
                    native::RAR_VOL_ASK => -1,
                    // Next volume found, 0 means continue
                    _ => 0,
                }
            }
            native::UCM_PROCESSDATA => {
                let raw_slice = std::ptr::slice_from_raw_parts(p1 as *const u8, p2 as _);
                M::process_data(&mut user_data.0, unsafe { &*raw_slice as &_ });
                0
            }
            _ => 0,
        }
    }

    fn process_file_raw(
        handle: &Handle,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> UnrarResult<M::Output> {
        let mut user_data: Userdata<M::Output> = Default::default();
        unsafe {
            native::RARSetCallback(
                handle.0.as_ptr(),
                Some(Self::callback),
                &mut user_data as *mut _ as native::LPARAM,
            );
        }
        let process_result = Code::from(unsafe {
            native::RARProcessFileW(
                handle.0.as_ptr(),
                M::OPERATION as i32,
                path.map(|path| path.as_ptr() as *const _)
                    .unwrap_or(std::ptr::null()),
                file.map(|file| file.as_ptr() as *const _)
                    .unwrap_or(std::ptr::null()),
            )
        })
        .unwrap();
        match process_result {
            Code::Success => Ok(user_data.0),
            Code::EOpen | _ => Err(UnrarError::from(process_result, When::Process)),
        }
    }
}

bitflags::bitflags! {
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
pub struct FileHeader {
    pub filename: PathBuf,
    pub flags: EntryFlags,
    pub unpacked_size: usize,
    pub file_crc: u32,
    pub file_time: u32,
    pub method: u32,
    pub file_attr: u32,
    pub next_volume: Option<PathBuf>,
}

impl FileHeader {
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

impl fmt::Display for FileHeader {
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

impl From<native::HeaderDataEx> for FileHeader {
    fn from(header: native::HeaderDataEx) -> Self {
        let filename =
            unsafe { WideCString::from_ptr_truncate(header.filename_w.as_ptr() as *const _, 1024) };

        FileHeader {
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

#[derive(Debug)]
pub struct Entry<T> {
    pub header: FileHeader,
    pub data: T,
}

fn unpack_unp_size(unp_size: c_uint, unp_size_high: c_uint) -> usize {
    ((unp_size_high as usize) << (8 * std::mem::size_of::<c_uint>())) | (unp_size as usize)
}

#[cfg(test)]
mod tests {
    #[test]
    fn combine_size() {
        use super::unpack_unp_size;
        let (high, low) = (1u32, 1464303715u32);
        assert_eq!(unpack_unp_size(low, high), 5759271011);
    }
}
