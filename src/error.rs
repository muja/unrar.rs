#![allow(missing_docs)]

use thiserror::Error;
use unrar_sys as native;

macro_rules! rarerror {
    ($name:ident$(#[$($tags:tt)*]$variant_name:ident=$variant_code:path),+$(,)?) => {
        #[repr(u32)]
        #[derive(PartialEq, Eq, Error, Debug, Clone, Copy)]
        pub enum $name {
            #[error("Unknown error")]
            Unknown = native::ERAR_UNKNOWN,
            $(
                #[$($tags)*]
                $variant_name = $variant_code
            ),+
        }
        impl From<u32> for $name {
            fn from(code: u32) -> Self {
                use $name::*;
                match code {
                    $(
                        $variant_code => $variant_name,
                    )+
                    _ => Unknown,
                }
            }
        }
    }
}

rarerror!(OpenError
    #[error("Archive header damaged")]
    ArchiveHeaderDamaged = native::ERAR_BAD_DATA,
    #[error("Unknown encryption")]
    UnkownEncryption = native::ERAR_UNKNOWN_FORMAT,
    #[error("Wrong password was specified")]
    BadPassword = native::ERAR_BAD_PASSWORD,
    #[error("Not enough memory")]
    NoMemory = native::ERAR_NO_MEMORY,
    #[error("Could not open archive")]
    Open = native::ERAR_EOPEN,
);

rarerror!(HeaderError 
    #[error("End of archive")]
    EndArchive = native::ERAR_END_ARCHIVE,
    #[error("File header damaged")]
    BrokenHeader = native::ERAR_BAD_DATA,
    #[error("Password was not provided for encrypted file header")]
    MissingPassword = native::ERAR_MISSING_PASSWORD,
    #[error("Could not open next volume")]
    VolumeOpen = native::ERAR_EOPEN,
);

rarerror!(ProcessError
    #[error("File CRC error")]
    FileCRC = native::ERAR_BAD_DATA,
    #[error("Unknown archive format")]
    UnknownFormat = native::ERAR_UNKNOWN_FORMAT,
    #[error("Could not open next volume")]
    VolumeOpen = native::ERAR_EOPEN,
    #[error("Read error")]
    Read = native::ERAR_EREAD,
    #[error("Write error")]
    Write = native::ERAR_EWRITE,
    #[error("Not enough memory")]
    NoMemory = native::ERAR_NO_MEMORY,
    #[error("Cannot open file source for reference record")]
    EReference = native::ERAR_EREFERENCE,
    #[error("Entered password is invalid")]
    BadPassword = native::ERAR_BAD_PASSWORD,
    #[error("Password was not provided for encrypted file header")]
    MissingPassword = native::ERAR_MISSING_PASSWORD,
);

rarerror!(ExtractError
    #[error("File CRC error")]
    FileCRC = native::ERAR_BAD_DATA,
    #[error("Unknown archive format")]
    UnknownFormat = native::ERAR_UNKNOWN_FORMAT,
    #[error("Could not open next volume")]
    VolumeOpen = native::ERAR_EOPEN,
    #[error("Read error")]
    Read = native::ERAR_EREAD,
    #[error("Write error")]
    Write = native::ERAR_EWRITE,
    #[error("Not enough memory")]
    NoMemory = native::ERAR_NO_MEMORY,
    #[error("Cannot open file source for reference record")]
    EReference = native::ERAR_EREFERENCE,
    #[error("Entered password is invalid")]
    BadPassword = native::ERAR_BAD_PASSWORD,
    #[error("Password was not provided for encrypted file header")]
    MissingPassword = native::ERAR_MISSING_PASSWORD,
    #[error("File create error")]
    FileCreate = native::ERAR_ECREATE,
    #[error("File close error")]
    FileClose = native::ERAR_ECLOSE,
);

#[derive(PartialEq, Eq, Error, Debug, Clone, Copy)]
pub enum IterateError {
    #[error(transparent)]
    Header(#[from] HeaderError),
    #[error(transparent)]
    Skip(#[from] ProcessError),
    
}

pub trait RarError: std::fmt::Display {}
impl RarError for OpenError {}
impl RarError for HeaderError {}
impl RarError for ProcessError {}
impl RarError for ExtractError {}
impl RarError for IterateError {}
