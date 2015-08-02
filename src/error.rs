use native;
use std::result::Result;
use num::FromPrimitive;

enum_from_primitive! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, PartialEq)]
pub struct UnrarError {
    pub code: Code,
    pub when: When,
    pub data: Option<String>
}

impl UnrarError {
    pub fn new(code: Code, when: When, data: String) -> Self {
        UnrarError { code: code, when: when, data: Some(data) }
    }

    pub fn from(code: Code, when: When) -> Self {
        UnrarError { code: code, when: when, data: None }
    }
}

pub type UnrarResult<T> = Result<T, UnrarError>;
