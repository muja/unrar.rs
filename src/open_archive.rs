use super::error::*;
use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_int, c_uint};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use widestring::WideCString;
use unrar_sys as native;

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

/// Volume information on the file that was *initially* opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeInfo {
    /// the *initially* opened file is a single-part archive
    None,
    /// the *initially* opened file is the first volume in a multipart archive
    First,
    /// the *initially* opened file is any volume but the first in a multipart archive
    Subsequent,
}

#[derive(Debug)]
struct Handle(NonNull<native::Handle>);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { native::RARCloseArchive(self.0.as_ptr() as *const _) };
    }
}

/// An open RAR archive that can be read or processed.
///
/// See the [OpenArchive chapter](index.html#openarchive) for more information.
#[derive(Debug)]
pub struct OpenArchive<M: OpenMode, C: Cursor> {
    handle: Handle,
    flags: ArchiveFlags,
    last_error: Option<rar::Error>,
    extra: C,
    marker: std::marker::PhantomData<M>,
}
type Userdata<T> = (T, Option<WideCString>);

mod private {
    use super::native;
    pub trait Sealed {}
    impl Sealed for super::CursorBeforeHeader {}
    impl Sealed for super::CursorBeforeFile {}
    impl Sealed for super::List {}
    impl Sealed for super::ListSplit {}
    impl Sealed for super::Process {}

    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Operation {
        Skip = native::RAR_SKIP,
        Test = native::RAR_TEST,
        Extract = native::RAR_EXTRACT,
    }

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OpenModeValue {
        Extract = native::RAR_OM_EXTRACT,
        List = native::RAR_OM_LIST,
        ListIncSplit = native::RAR_OM_LIST_INCSPLIT,
    }
}

/// Type parameter for OpenArchive denoting a `read_header` operation must follow next.
///
/// See the chapter [OpenArchive: Cursors](index.html#openarchive-cursors) for more information.
#[derive(Debug)]
pub struct CursorBeforeHeader;
/// Type parameter for OpenArchive denoting a `process_file` operation must follow next.
///
/// See the chapter [OpenArchive: Cursors](index.html#openarchive-cursors) for more information.
#[derive(Debug)]
pub struct CursorBeforeFile {
    header: FileHeader,
}

/// The Cursor trait enables archives to keep track of their state.
///
/// See the chapter [OpenArchive: Cursors](index.html#openarchive-cursors) for more information.
pub trait Cursor: private::Sealed {}
impl Cursor for CursorBeforeHeader {}
impl Cursor for CursorBeforeFile {}

/// An OpenMode for processing RAR archive entries.
///
/// Process allows more sophisticated operations in the `ProcessFile` step.
#[derive(Debug)]
pub struct Process;
#[derive(Debug)]
/// An OpenMode for listing RAR archive entries.
///
/// List mode will list all entries. The payload itself cannot be processed and instead can only
/// be skipped over. This will yield one header per individual file, regardless of how many parts
/// the file is split across.
pub struct List;
/// An OpenMode for listing RAR archive entries.
///
/// ListSplit mode will list all entries. The payload itself cannot be processed and instead can
/// only be skipped over. This will yield one header per individual file per part if the file is
/// split across multiple parts. The [`FileHeader::is_split`] method will return true in that case.
#[derive(Debug)]
pub struct ListSplit;

/// Mode with which the archive should be opened.
///
/// Possible modes are:
///
///    - [`List`](struct.List.html)
///    - [`ListSplit`](struct.ListSplit.html)
///    - [`Process`](struct.Process.html)
pub trait OpenMode: private::Sealed + std::fmt::Debug {
    const VALUE: private::OpenModeValue;
}
impl OpenMode for Process {
    const VALUE: private::OpenModeValue = private::OpenModeValue::Extract;
}
impl OpenMode for List {
    const VALUE: private::OpenModeValue = private::OpenModeValue::List;
}
impl OpenMode for ListSplit {
    const VALUE: private::OpenModeValue = private::OpenModeValue::ListIncSplit;
}

impl<Mode: OpenMode, C: Cursor> OpenArchive<Mode, C> {
    /// is the archive locked
    pub fn is_locked(&self) -> bool {
        self.flags.contains(ArchiveFlags::LOCK)
    }

    /// are the archive headers encrypted
    pub fn has_encrypted_headers(&self) -> bool {
        self.flags.contains(ArchiveFlags::ENC_HEADERS)
    }

    /// does the archive have a recovery record
    pub fn has_recovery_record(&self) -> bool {
        self.flags.contains(ArchiveFlags::RECOVERY)
    }

    /// does the archive have comments
    pub fn has_comment(&self) -> bool {
        self.flags.contains(ArchiveFlags::COMMENT)
    }

    /// is the archive solid (all files in a single compressed block).
    pub fn is_solid(&self) -> bool {
        self.flags.contains(ArchiveFlags::SOLID)
    }

    /// Volume information on the file that was *initially* opened.
    ///
    /// returns
    ///   - `VolumeInfo::None` if the opened file is a single-part archive
    ///   - `VolumeInfo::First` if the opened file is the first volume in a multipart archive
    ///   - `VolumeInfo::Subsequent` if the opened file is any other volume in a multipart archive
    ///
    /// Note that this value *never* changes from `First` to `Subsequent` by advancing to a
    /// different volume.
    pub fn volume_info(&self) -> VolumeInfo {
        if self.flags.contains(ArchiveFlags::FIRST_VOLUME) {
            VolumeInfo::First
        } else if self.flags.contains(ArchiveFlags::VOLUME) {
            VolumeInfo::Subsequent
        } else {
            VolumeInfo::None
        }
    }

    /// unsets the `damaged` flag so that `Iterator` will not refuse to yield elements.
    ///
    /// Normally, when an error is returned during iteration, the archive remembers this
    /// so that subsequent calls to `next` return `None` immediately. This is to prevent
    /// the same error from recurring over and over again, leading to endless loops in programs
    /// that might not have considered this. However, maybe there are errors that can be recovered
    /// from? That's where this method might come in handy if you really know what you're doing.
    /// However, should that be the case, I urge you to submit an issue / PR with an archive where
    /// the recoverable error can be reproduced so I can exclude that case from "irrecoverable
    /// errors" (currently all errors).
    ///
    /// Use at your own risk. Might be removed in future releases if somehow it can be verified
    /// which errors are recoverable and which are not.
    ///
    /// # Example how you *might* use this
    ///
    /// ```no_run
    /// use unrar::{Archive, error::RarError};
    ///
    /// let mut archive = Archive::new("corrupt.rar").open_for_listing().expect("archive error");
    /// loop {
    ///     let mut error = None;
    ///     for result in &mut archive {
    ///         match result {
    ///             Ok(entry) => println!("{entry}"),
    ///             Err(e) => error = Some(e),
    ///         }
    ///     }
    ///     match error {
    ///         // your special recoverable error, please submit a PR with reproducible archive
    ///         Some(RarError::Unknown) => archive.force_heal(),
    ///         Some(e) => panic!("irrecoverable error: {e}"),
    ///         None => break,
    ///     }
    /// }
    /// ```
    pub fn force_heal(&mut self) {
        self.last_error = None;
    }
}

impl<Mode: OpenMode> OpenArchive<Mode, CursorBeforeHeader> {
    pub(crate) fn new(
        filename: &Path,
        password: Option<&[u8]>,
        recover: Option<&mut Option<Self>>,
    ) -> Result<Self> {
        #[cfg(target_os = "linux")]
        // on Linux there is an issue with unicode filenames when using wide strings
        // so we use non-wide strings. See https://github.com/muja/unrar.rs/issues/44
        let filename = CString::new(filename.as_os_str().as_encoded_bytes())?;
        #[cfg(not(target_os = "linux"))]
        let filename = WideCString::from_os_str(&filename)?;

        let mut data =
            native::OpenArchiveDataEx::new(filename.as_ptr() as *const _, Mode::VALUE as u32);
        let handle =
            NonNull::new(unsafe { native::RAROpenArchiveEx(&mut data as *mut _) } as *mut _);

        if let Some(handle) = handle {
            if let Some(pw) = password {
                let cpw = CString::new(pw)?;
                unsafe { native::RARSetPassword(handle.as_ptr(), cpw.as_ptr() as *const _) }
            }
        }

        let arc = handle.and_then(|handle| {
            Some(OpenArchive {
                handle: Handle(handle),
                last_error: None,
                flags: ArchiveFlags::from_bits(data.flags).unwrap(),
                extra: CursorBeforeHeader,
                marker: std::marker::PhantomData,
            })
        });

        match (arc, data.open_result) {
            (Some(arc), native::ERAR_SUCCESS) => Ok(arc),
            (arc, code) => {
                recover.and_then(|recover| arc.and_then(|arc| recover.replace(arc)));
                Err(rar::Error::at_open(code).unwrap().into())
            }
        }
    }

    /// reads the next header of the underlying archive. The resulting OpenArchive will
    /// be in "ProcessFile" mode, i.e. the file corresponding to the header (that has just
    /// been read via this method call) will have to be read. Also contains header data
    /// via [`archive.entry()`](OpenArchive::entry).
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let archive = unrar::Archive::new("data/version.rar").open_for_listing().unwrap().read_header();
    /// assert!(archive.as_ref().is_ok_and(Option::is_some));
    /// let archive = archive.unwrap().unwrap();
    /// assert_eq!(archive.entry().filename.as_os_str(), "VERSION");
    /// ```
    #[tracing::instrument(level = "debug", ret)]
    pub fn read_header(self) -> rar::Result<Option<OpenArchive<Mode, CursorBeforeFile>>> {
        Ok(read_header(&self.handle)?.map(|entry| OpenArchive {
            extra: CursorBeforeFile { header: entry },
            last_error: self.last_error,
            handle: self.handle,
            flags: self.flags,
            marker: std::marker::PhantomData,
        }))
    }

    /// Reads the current header and returns a handle to the corresponding entry.
    ///
    /// This is an alternative, more ergonomic API to calling `archive.read_header` and
    /// `archive.{read,skip,extract*}` directly (alternatingly) and reassigning the `archive`
    /// variable every time. Instead, using this method, extracting could look like this:
    ///
    /// ```
    /// # let filename = "data/version.rar";
    /// // set filename, in this example we use the test file data/version.rar
    /// let mut archive = unrar::Archive::new(filename).open_for_processing().unwrap();
    /// while let Some(entry) = archive.try_next().unwrap() {
    ///     println!("currently processing file {}", entry.header());
    ///     if entry.header().filename.as_os_str() == "VERSION" {
    ///         let bytes = entry.read().unwrap();
    ///         let version = String::from_utf8_lossy(&bytes);
    ///         assert_eq!(version, "unrar-0.4.0");
    ///     } else {
    ///         entry.skip().unwrap();
    ///         # panic!("")
    ///     }
    /// }
    /// ```
    ///
    /// This API uses the `streaming iterator` pattern to return a handle to the internal
    /// entry which has to be processed before calling this method again (either `skip`, `process`).
    /// The returned handle can be used to inspect the header and process the file accordingly.
    /// If it is ignored, its destructor will try to `skip()` over the entry. Should that operation
    /// fail, the error will be reported using `tracing::error!` and returned in the *next* call to
    /// this method. Since linear/relevant types are not implemented in Rust, this is a best
    /// effort to ensure errors will be handled properly. If you want none of these shenanigans,
    /// you should call `skip()` directly.
    pub fn try_next<'a>(&'a mut self) -> rar::Result<Option<ProcessHandle<'a, Mode>>> {
        if self.last_error.is_some() {
            Err(self.last_error.take().unwrap())?
        }
        read_header(&self.handle).map(|opt| opt.map(|header| ProcessHandle(false, header, self)))
    }
}

macro_rules! iterator_impl {
    ($x: ident) => {
        impl Iterator for OpenArchive<$x, CursorBeforeHeader> {
            type Item = rar::Result<FileHeader>;

            #[tracing::instrument(level = "debug", ret)]
            fn next(&mut self) -> Option<Self::Item> {
                if self.last_error.is_some() {
                    return None;
                }
                match read_header(&self.handle) {
                    Ok(Some(header)) => {
                        match Internal::<Skip>::process_file_raw(self.handle.0.as_ptr(), None, None)
                        {
                            Ok(_) => Some(Ok(header)),
                            Err(s) => {
                                self.last_error = Some(s);
                                Some(Err(s))
                            }
                        }
                    }
                    Ok(None) => None,
                    Err(s) => {
                        self.last_error = Some(s);
                        Some(Err(s))
                    }
                }
            }
        }
    };
}
iterator_impl!(List);
iterator_impl!(ListSplit);

impl<M: OpenMode> OpenArchive<M, CursorBeforeFile> {
    /// returns the file header for the file that follows which is to be processed next.
    pub fn entry(&self) -> &FileHeader {
        &self.extra.header
    }

    /// skips over the next file, not doing anything with it.
    pub fn skip(self) -> rar::Result<OpenArchive<M, CursorBeforeHeader>> {
        self.process_file::<Skip>(None, None)
    }

    #[tracing::instrument(level = "debug", ret)]
    fn process_file<PM: ProcessMode>(
        self,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> rar::Result<OpenArchive<M, CursorBeforeHeader>> {
        Ok(self.process_file_x::<PM>(path, file)?.1)
    }

    fn process_file_x<PM: ProcessMode>(
        self,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> rar::Result<(PM::Output, OpenArchive<M, CursorBeforeHeader>)> {
        let result = Ok((
            Internal::<PM>::process_file_raw(self.handle.0.as_ptr(), path, file)?,
            OpenArchive {
                extra: CursorBeforeHeader,
                last_error: self.last_error,
                handle: self.handle,
                flags: self.flags,
                marker: std::marker::PhantomData,
            },
        ));
        result
    }
}

impl OpenArchive<Process, CursorBeforeFile> {
    /// Reads the underlying file into a `Vec<u8>`
    /// Returns the data as well as the owned Archive that can be processed further.
    pub fn read(self) -> rar::Result<(Vec<u8>, OpenArchive<Process, CursorBeforeHeader>)> {
        Ok(self.process_file_x::<ReadToVec>(None, None)?)
    }

    /// Test the file without extracting it
    pub fn test(self) -> rar::Result<OpenArchive<Process, CursorBeforeHeader>> {
        Ok(self.process_file::<Test>(None, None)?)
    }

    /// Extracts the file into the current working directory
    /// 
    /// Returns the OpenArchive for further processing
    pub fn extract(self) -> rar::Result<OpenArchive<Process, CursorBeforeHeader>> {
        self.process_file::<Extract>(None, None)
    }

    /// Extracts the file into the specified directory.  
    /// Returns the OpenArchive for further processing
    /// 
    /// # Errors
    ///
    /// `NulError` if `dest` contains nul characters.
    pub fn extract_with_base<P: AsRef<Path>>(
        self,
        base: P,
    ) -> Result<OpenArchive<Process, CursorBeforeHeader>> {
        let wdest = WideCString::from_os_str(base.as_ref())?;
        self.process_file::<Extract>(Some(&wdest), None)
            .map_err(Error::RarError)
    }

    /// Extracts the file into the specified file.
    /// Returns the OpenArchive for further processing
    ///
    /// # Errors
    ///
    /// `NulError` if `dest` contains nul characters.
    pub fn extract_to<P: AsRef<Path>>(
        self,
        dest: P,
    ) -> Result<OpenArchive<Process, CursorBeforeHeader>> {
        let wdest = WideCString::from_os_str(dest.as_ref())?;
        self.process_file::<Extract>(None, Some(&wdest))
            .map_err(Error::RarError)
    }
}

pub struct ProcessHandle<'a, Mode: OpenMode>(
    bool,
    FileHeader,
    &'a mut OpenArchive<Mode, CursorBeforeHeader>,
);

impl<'a, Mode: OpenMode> ProcessHandle<'a, Mode> {
    pub fn header(&self) -> &FileHeader {
        &self.1
    }

    pub fn skip(self) -> rar::Result<()> {
        self.process::<Skip>(None, None)
    }

    fn process<M: ProcessMode>(
        mut self,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> rar::Result<M::Output> {
        self.0 = true;
        Internal::<M>::process_file_raw(self.2.handle.0.as_ptr(), path, file)
    }
}

impl<'a> ProcessHandle<'a, Process> {
    pub fn read(self) -> rar::Result<Vec<u8>> {
        self.process::<ReadToVec>(None, None)
    }

    /// extracts into current working directory
    pub fn extract_cwd(self) -> rar::Result<()> {
        self.process::<Extract>(None, None)
    }

    /// extracts under the specified path
    pub fn extract_with_base<P: AsRef<Path>>(self, base: P) -> Result<()> {
        let wdest = WideCString::from_os_str(base.as_ref())?;
        self.process::<Extract>(Some(&wdest), None)
            .map_err(Error::RarError)
    }
}

impl<'a, Mode: OpenMode> Drop for ProcessHandle<'a, Mode> {
    fn drop(&mut self) {
        // process handle which must be processed
        if !self.0 {
            // !self.0 means this was not processed so we try to skip the entry in the destructor
            let result = Internal::<Skip>::process_file_raw(self.2.handle.0.as_ptr(), None, None);
            if let Err(e) = result {
                tracing::error!("error skipping file in destructor ProcessHandle::drop: {e}");
                self.2.last_error = Some(e);
            }
        }
    }
}

#[tracing::instrument(level = "debug", ret)]
fn read_header(handle: &Handle) -> rar::Result<Option<FileHeader>> {
    let mut userdata: Userdata<<Skip as ProcessMode>::Output> = Default::default();
    unsafe {
        native::RARSetCallback(
            handle.0.as_ptr(),
            Some(Internal::<Skip>::callback),
            &mut userdata as *mut _ as native::LPARAM,
        );
    }

    let mut header = native::HeaderDataEx::default();
    tracing::trace!("reading header....");
    let read_result = unsafe { native::RARReadHeaderEx(handle.0.as_ptr(), &mut header as *mut _) };
    tracing::trace!("read_result: {read_result:?}");
    match read_result {
        native::ERAR_SUCCESS => Ok(Some(header.into())),
        native::ERAR_END_ARCHIVE => Ok(None),
        code => Err(rar::Error::at_header(code).unwrap()),
    }
}

#[derive(Debug)]
struct Skip;
#[derive(Debug)]
struct ReadToVec;
#[derive(Debug)]
struct Extract;
#[derive(Debug)]
struct Test;

trait ProcessMode: core::fmt::Debug {
    const OPERATION: private::Operation;
    type Output: core::fmt::Debug + std::default::Default;

    fn process_data(data: &mut Self::Output, other: &[u8]);
}
impl ProcessMode for Skip {
    const OPERATION: private::Operation = private::Operation::Skip;
    type Output = ();

    fn process_data(_: &mut Self::Output, _: &[u8]) {}
}
impl ProcessMode for ReadToVec {
    const OPERATION: private::Operation = private::Operation::Test;
    type Output = Vec<u8>;

    fn process_data(my: &mut Self::Output, other: &[u8]) {
        my.extend_from_slice(other);
    }
}
impl ProcessMode for Extract {
    const OPERATION: private::Operation = private::Operation::Extract;
    type Output = ();

    fn process_data(_: &mut Self::Output, _: &[u8]) {}
}
impl ProcessMode for Test {
    const OPERATION: private::Operation = private::Operation::Test;
    type Output = ();

    fn process_data(_: &mut Self::Output, _: &[u8]) {}
}

struct Internal<M: ProcessMode>(std::marker::PhantomData<M>);

impl<M: ProcessMode> Internal<M> {
    #[tracing::instrument(level = "trace", ret)]
    extern "C" fn callback(
        msg: native::UINT,
        user_data: native::LPARAM,
        p1: native::LPARAM,
        p2: native::LPARAM,
    ) -> c_int {
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
        handle: *const native::Handle,
        path: Option<&WideCString>,
        file: Option<&WideCString>,
    ) -> rar::Result<M::Output> {
        tracing::debug!("ENTER");
        let mut user_data: Userdata<M::Output> = Default::default();
        unsafe {
            native::RARSetCallback(
                handle,
                Some(Self::callback),
                &mut user_data as *mut _ as native::LPARAM,
            );
        }
        let process_result = unsafe {
            native::RARProcessFileW(
                handle,
                M::OPERATION as i32,
                path.map(|path| path.as_ptr() as *const _)
                    .unwrap_or(std::ptr::null()),
                file.map(|file| file.as_ptr() as *const _)
                    .unwrap_or(std::ptr::null()),
            )
        };
        match process_result {
            native::ERAR_SUCCESS => Ok(user_data.0),
            code => Err(rar::Error::at_process(code).unwrap()),
        }
    }
}

bitflags::bitflags! {
    struct EntryFlags: u32 {
        const SPLIT_BEFORE = 0x1;
        const SPLIT_AFTER = 0x2;
        const ENCRYPTED = 0x4;
        // const RESERVED = 0x8;
        const SOLID = 0x10;
        const DIRECTORY = 0x20;
    }
}

/// Metadata for an entry in a RAR archive
///
/// Created using the read_header methods in an OpenArchive, contains
/// information for the file that follows which is to be processed next.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub filename: PathBuf,
    flags: EntryFlags,
    pub unpacked_size: u64,
    pub file_crc: u32,
    pub file_time: u32,
    pub method: u32,
    pub file_attr: u32,
}

impl FileHeader {
    /// is this entry split across multiple volumes.
    ///
    /// Will also work in open mode [`List`]
    pub fn is_split(&self) -> bool {
        self.flags.contains(EntryFlags::SPLIT_BEFORE)
            || self.flags.contains(EntryFlags::SPLIT_AFTER)
    }

    /// is this entry split across multiple volumes, starting here
    ///
    /// Will also work in open mode [`List`]
    pub fn is_split_after(&self) -> bool {
        self.flags.contains(EntryFlags::SPLIT_AFTER)
    }

    /// is this entry split across multiple volumes, starting here
    ///
    /// Will always return false in open mode [`List`][^1].
    ///
    /// [^1]: this claim is not proven, however, the DLL seems to always skip
    /// files where this flag would have been set.
    pub fn is_split_before(&self) -> bool {
        self.flags.contains(EntryFlags::SPLIT_BEFORE)
    }

    /// is this entry a directory
    pub fn is_directory(&self) -> bool {
        self.flags.contains(EntryFlags::DIRECTORY)
    }

    /// is this entry encrypted
    pub fn is_encrypted(&self) -> bool {
        self.flags.contains(EntryFlags::ENCRYPTED)
    }

    /// is this entry a file
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
        }
    }
}

fn unpack_unp_size(unp_size: c_uint, unp_size_high: c_uint) -> u64 {
    ((unp_size_high as u64) << (8 * std::mem::size_of::<c_uint>())) | (unp_size as u64)
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
