use native;
use std::result::Result;
use std::ffi;
use num::FromPrimitive;

enum_from_primitive! {
    #[derive(PartialEq, Eq, Debug)]
    #[repr(i32)]
    pub enum Code {
        Success = native::ERAR_SUCCESS,
        EndArchive = native::ERAR_END_ARCHIVE,
        NoMemory = native::ERAR_NO_MEMORY,
        BadData = native::ERAR_BAD_DATA,
        BadArchive = native::ERAR_BAD_ARCHIVE,
        UnknownFormat = native::ERAR_UNKNOWN_FORMAT,
        EOpen = native::ERAR_EOPEN,
        ECreate = native::ERAR_ECREATE,
        EClose = native::ERAR_ECLOSE,
        ERead = native::ERAR_EREAD,
        EWrite = native::ERAR_EWRITE,
        SmallBuf = native::ERAR_SMALL_BUF,
        Unknown = native::ERAR_UNKNOWN,
        MissingPassword = native::ERAR_MISSING_PASSWORD,
        EReference = native::ERAR_EREFERENCE,
        BadPassword = native::ERAR_BAD_PASSWORD
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum When {
    Open,
    Read,
    Process
}

impl Code {
    pub fn from(code: u32) -> Option<Self> {
        Code::from_u32(code)
    }
}

impl From<ffi::NulError> for UnrarError {
    fn from(error: ffi::NulError) -> UnrarError {
        UnrarError::Nul(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum UnrarError {
    Native(Code, When),
    Nul(ffi::NulError)
}

impl UnrarError {
    pub fn from(code: Code, when: When) -> Self {
        UnrarError::Native(code, when)
    }
}

pub type UnrarResult<T> = Result<T, UnrarError>;
