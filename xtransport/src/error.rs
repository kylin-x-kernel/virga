// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidMagic,
    InvalidVersion,
    CrcMismatch,
    UnexpectedEof,
    InvalidPacket,
    WriteZero,
    Interrupted,
    Other,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.kind {
            ErrorKind::UnexpectedEof => "Unexpected end of file",
            ErrorKind::WriteZero => "Write zero bytes",
            ErrorKind::InvalidMagic => "Invalid magic number",
            ErrorKind::CrcMismatch => "CRC checksum mismatch",
            ErrorKind::InvalidPacket => "Invalid packet",
            ErrorKind::InvalidVersion => "Invalid protocol version",
            ErrorKind::Interrupted => "Operation interrupted",
            ErrorKind::Other => "Other error",
        };
        f.write_str(msg)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(err: Error) -> std::io::Error {
        let kind = match err.kind {
            ErrorKind::UnexpectedEof => std::io::ErrorKind::UnexpectedEof,
            ErrorKind::WriteZero => std::io::ErrorKind::WriteZero,
            ErrorKind::Interrupted => std::io::ErrorKind::Interrupted,
            _ => std::io::ErrorKind::Other,
        };
        std::io::Error::new(kind, err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
