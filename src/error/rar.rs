use thiserror::Error;
use unrar_sys as native;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(PartialEq, Eq, Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("Archive header damaged")]
    ArchiveHeaderDamaged,
    #[error("File header damaged")]
    FileHeaderDamaged,
    #[error("File CRC error")]
    FileCRCError,
    #[error("Unknown encryption")]
    UnkownEncryption,
    #[error("Could not open next volume")]
    NextVolumeNotFound,
    #[error("Unknown archive format")]
    UnknownFormat,
    #[error("Could not open archive")]
    EOpen,
    #[error("Not enough memory")]
    NoMemory,
    #[error("Not a RAR archive")]
    BadArchive,
    #[error("Could not create file")]
    ECreate,
    #[error("Could not close file")]
    EClose,
    #[error("Read error")]
    ERead,
    #[error("Write error")]
    EWrite,
    #[error("Archive comment was truncated to fit to buffer")]
    SmallBuf,
    #[error("Password for encrypted archive not specified")]
    MissingPassword,
    #[error("Wrong password was specified")]
    BadPassword,
    #[error("Unknown error")]
    Unknown,
    // From the UnRARDLL docs:
    // When attempting to unpack a reference record (see RAR -oi switch),
    // source file for this reference was not found.
    // Entire archive needs to be unpacked to properly create file references.
    // This error is returned when attempting to unpack the reference
    // record without its source file.
    #[error("Cannot open file source for reference record")]
    EReference,
}

impl Error {
    pub(crate) fn at_open(code: u32) -> Option<Self> {
        Self::from(code, When::Open)
    }

    pub(crate) fn at_header(code: u32) -> Option<Self> {
        Self::from(code, When::Header)
    }
    
    pub(crate) fn at_process(code: u32) -> Option<Self> {
        Self::from(code, When::Process)
    }
    
    fn from(code: u32, when: When) -> Option<Self> {
        use Error::*;
        match (code, when) {
            (native::ERAR_BAD_DATA, When::Open) => Some(ArchiveHeaderDamaged),
            (native::ERAR_BAD_DATA, When::Header) => Some(FileHeaderDamaged),
            (native::ERAR_BAD_DATA, When::Process) => Some(FileCRCError),
            (native::ERAR_UNKNOWN_FORMAT, When::Open) => Some(UnkownEncryption),
            (native::ERAR_EOPEN, When::Process) => Some(NextVolumeNotFound),
            (native::ERAR_NO_MEMORY, _) => Some(NoMemory),
            (native::ERAR_BAD_ARCHIVE, _) => Some(BadArchive),
            (native::ERAR_UNKNOWN_FORMAT, _) => Some(UnknownFormat),
            (native::ERAR_EOPEN, _) => Some(EOpen),
            (native::ERAR_ECREATE, _) => Some(ECreate),
            (native::ERAR_ECLOSE, _) => Some(EClose),
            (native::ERAR_EREAD, _) => Some(ERead),
            (native::ERAR_EWRITE, _) => Some(EWrite),
            (native::ERAR_SMALL_BUF, _) => Some(SmallBuf),
            (native::ERAR_UNKNOWN, _) => Some(Unknown),
            (native::ERAR_MISSING_PASSWORD, _) => Some(MissingPassword),
            (native::ERAR_EREFERENCE, _) => Some(EReference),
            (native::ERAR_BAD_PASSWORD, _) => Some(BadPassword),
            _ => None,
        }
    }
}

enum When {
    Open,
    Header,
    Process,
}
