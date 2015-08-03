extern crate libc;

use libc::{c_int, c_uint, wchar_t, c_long, c_char, c_void, c_uchar};

// ----------------- CONSTANTS ----------------- //

pub const ERAR_SUCCESS: c_int = 0;
pub const ERAR_END_ARCHIVE: c_int = 10;
pub const ERAR_NO_MEMORY: c_int = 11;
pub const ERAR_BAD_DATA: c_int = 12;
pub const ERAR_BAD_ARCHIVE: c_int = 13;
pub const ERAR_UNKNOWN_FORMAT: c_int = 14;
pub const ERAR_EOPEN: c_int = 15;
pub const ERAR_ECREATE: c_int = 16;
pub const ERAR_ECLOSE: c_int = 17;
pub const ERAR_EREAD: c_int = 18;
pub const ERAR_EWRITE: c_int = 19;
pub const ERAR_SMALL_BUF: c_int = 20;
pub const ERAR_UNKNOWN: c_int = 21;
pub const ERAR_MISSING_PASSWORD: c_int = 22;
pub const ERAR_EREFERENCE: c_int = 23;
pub const ERAR_BAD_PASSWORD: c_int = 24;

pub const RAR_OM_LIST: c_uint = 0;
pub const RAR_OM_EXTRACT: c_uint = 1;
pub const RAR_OM_LIST_INCSPLIT: c_uint = 2;

pub const RAR_SKIP: c_int = 0;
pub const RAR_TEST: c_int = 1;
pub const RAR_EXTRACT: c_int = 2;

pub const RAR_VOL_ASK: c_long = 0;
pub const RAR_VOL_NOTIFY: c_long = 1;

pub const RAR_HASH_NONE: c_uint = 0;
pub const RAR_HASH_CRC32: c_uint = 1;
pub const RAR_HASH_BLAKE2: c_uint = 2;

pub const RHDF_SPLITBEFORE: c_uint = 1 << 0; // 1, 0x1
pub const RHDF_SPLITAFTER: c_uint = 1 << 1; // 2, 0x2
pub const RHDF_ENCRYPTED: c_uint = 1 << 2; // 4, 0x4
// pub const RHDF_RESERVED: c_uint = 1 << 3; // 8, 0x8
pub const RHDF_SOLID: c_uint = 1 << 4; // 16, 0x10
pub const RHDF_DIRECTORY: c_uint = 1 << 5; // 32, 0x20

pub const UCM_CHANGEVOLUME: c_uint = 0;
pub const UCM_PROCESSDATA: c_uint = 1;
pub const UCM_NEEDPASSWORD: c_uint = 2;
pub const UCM_CHANGEVOLUMEW: c_uint = 3;
pub const UCM_NEEDPASSWORDW: c_uint = 4;

pub type ChangeVolProc = extern "C" fn(*mut c_char, c_int) -> c_int;
pub type ProcessDataProc = extern "C" fn(*mut c_uchar, c_int) -> c_int;
pub type Callback = extern "C" fn(
    c_uint,
    c_long,
    c_long,
    c_long
) -> c_int;

pub type Handle = *const c_void;

// ----------------- STRUCTS ----------------- //

#[repr(C)]
pub struct HeaderData {
    pub archive_name: [c_char; 260],
    pub filename: [c_char; 260],
    pub flags: c_uint,
    pub pack_size: c_uint,
    pub unp_size: c_uint,
    pub host_os: c_uint,
    pub file_crc: c_uint,
    pub file_time: c_uint,
    pub unp_ver: c_uint,
    pub method: c_uint,
    pub file_attr: c_uint,
    pub comment_buffer: *mut c_char,
    pub comment_buffer_size: c_uint,
    pub comment_size: c_uint,
    pub comment_state: c_uint
}

#[repr(C)]
pub struct HeaderDataEx {
    pub archive_name: [c_char; 1024],
    pub archive_name_w: [wchar_t; 1024],
    pub filename: [c_char; 1024],
    pub filename_w: [wchar_t; 1024],
    pub flags: c_uint,
    pub pack_size: c_uint,
    pub pack_size_high: c_uint,
    pub unp_size: c_uint,
    pub unp_size_high: c_uint,
    pub host_os: c_uint,
    pub file_crc: c_uint,
    pub file_time: c_uint,
    pub unp_ver: c_uint,
    pub method: c_uint,
    pub file_attr: c_uint,
    pub comment_buffer: *mut c_char,
    pub comment_buffer_size: c_uint,
    pub comment_size: c_uint,
    pub comment_state: c_uint,
    pub dict_size: c_uint,
    pub hash_type: c_uint,
    pub hash: [c_char; 32],
    pub reserved: [c_uint; 1014]
}

#[repr(C)]
pub struct OpenArchiveData {
    pub archive_name: *const c_char,
    pub open_mode: c_uint,
    pub open_result: c_uint,
    pub comment_buffer: *mut c_char,
    pub comment_buffer_size: c_uint,
    pub comment_size: c_uint,
    pub comment_state: c_uint,
}

#[repr(C)]
pub struct OpenArchiveDataEx {
    pub archive_name: *const c_char,
    pub archive_name_w: *const wchar_t,
    pub open_mode: c_uint,
    pub open_result: c_uint,
    pub comment_buffer: *mut c_char,
    pub comment_buffer_size: c_uint,
    pub comment_size: c_uint,
    pub comment_state: c_uint,
    pub flags: c_uint,
    pub callback: Option<Callback>,
    pub user_data: c_long,
    pub reserved: [c_uint; 28]
}

// ----------------- BINDINGS ----------------- //

#[link(name = "unrar", kind = "static")]
extern "C" {
    pub fn RAROpenArchive(data: *mut OpenArchiveData) -> Handle;

    pub fn RAROpenArchiveEx(data: *mut OpenArchiveDataEx) -> Handle;

    pub fn RARCloseArchive(handle: Handle) -> c_int;

    pub fn RARReadHeader(
        handle: Handle,
        header_data: *mut HeaderData
    ) -> c_int;

    pub fn RARReadHeaderEx(
        handle: Handle,
        header_data: *mut HeaderDataEx
    ) -> c_int;

    pub fn RARProcessFile(
        handle: Handle,
        operation: c_int,
        dest_path: *const c_char,
        dest_name: *const c_char
    ) -> c_int;

    pub fn RARProcessFileW(
        handle: Handle,
        operation: c_int,
        dest_path: *const wchar_t,
        dest_name: *const wchar_t
    ) -> c_int;

    pub fn RARSetCallback(
        handle: Handle,
        callback: Callback,
        user_data: c_long
    );

    pub fn RARSetChangeVolProc(
        handle: Handle,
        change_vol_proc: ChangeVolProc
    );

    pub fn RARSetProcessDataProc(
        handle: Handle,
        process_data_proc: ProcessDataProc
    );

    pub fn RARSetPassword(
        handle: Handle,
        password: *const c_char
    );

    pub fn RARGetDllVersion() -> c_int;
}

// ----------------- MINIMAL ABSTRACTIONS ----------------- //

impl Default for HeaderData {
    fn default() -> Self {
        HeaderData {
            archive_name: [0; 260],
            filename: [0; 260],
            flags: 0,
            pack_size: 0,
            unp_size: 0,
            host_os: 0,
            file_crc: 0,
            file_time: 0,
            unp_ver: 0,
            method: 0,
            file_attr: 0,
            comment_buffer: 0 as *mut _,
            comment_buffer_size: 0,
            comment_size: 0,
            comment_state: 0
        }
    }
}

impl Default for HeaderDataEx {
    fn default() -> Self {
        HeaderDataEx {
            archive_name: [0; 1024],
            archive_name_w: [0; 1024],
            filename: [0; 1024],
            filename_w: [0; 1024],
            flags: 0,
            pack_size: 0,
            pack_size_high: 0,
            unp_size: 0,
            unp_size_high: 0,
            host_os: 0,
            file_crc: 0,
            file_time: 0,
            unp_ver: 0,
            method: 0,
            file_attr: 0,
            comment_buffer: 0 as *mut _,
            comment_buffer_size: 0,
            comment_size: 0,
            comment_state: 0,
            dict_size: 0,
            hash_type: 0,
            hash: [0; 32],
            reserved: [0; 1014]
        }
    }
}

impl OpenArchiveData {
    pub fn new(archive: *const c_char, mode: c_uint) -> Self {
        Self::with_comment_buffer(archive, mode, 0 as *mut _, 0)
    }

    pub fn with_comment_buffer(
        archive_name: *const c_char,
        open_mode: c_uint,
        buffer: *mut c_char,
        buffer_size: c_uint
    ) -> Self {
        OpenArchiveData {
            archive_name: archive_name,
            open_mode: open_mode,
            comment_buffer: buffer,
            comment_buffer_size: buffer_size,
            // set by library:
            open_result: 0,
            comment_size: 0,
            comment_state: 0
        }
    }
}

impl Default for OpenArchiveDataEx {
    fn default() -> Self {
        OpenArchiveDataEx {
            archive_name: 0 as *const _,
            archive_name_w: 0 as *const _,
            open_mode: 0,
            open_result: 0,
            comment_buffer: 0 as *mut _,
            comment_buffer_size: 0,
            comment_size: 0,
            comment_state: 0,
            flags: 0,
            callback: None,
            user_data: 0,
            reserved: [0; 28]
        }
    }
}

// ----------------- TESTS ----------------- //

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        assert_eq!(unsafe{super::RARGetDllVersion()}, 7);
    }
}
