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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn error_kind_values() {
        let kinds = [
            ErrorKind::InvalidMagic,
            ErrorKind::InvalidVersion,
            ErrorKind::CrcMismatch,
            ErrorKind::UnexpectedEof,
            ErrorKind::InvalidPacket,
            ErrorKind::WriteZero,
            ErrorKind::Interrupted,
            ErrorKind::Other,
        ];
        for i in 0..kinds.len() {
            for j in (i + 1)..kinds.len() {
                assert_ne!(kinds[i], kinds[j]);
            }
        }
    }

    #[test]
    fn error_new_and_kind() {
        let err = Error::new(ErrorKind::CrcMismatch);
        assert_eq!(err.kind(), ErrorKind::CrcMismatch);
    }

    #[test]
    fn error_display_invalid_magic() {
        let err = Error::new(ErrorKind::InvalidMagic);
        assert_eq!(format!("{}", err), "Invalid magic number");
    }

    #[test]
    fn error_display_invalid_version() {
        let err = Error::new(ErrorKind::InvalidVersion);
        assert_eq!(format!("{}", err), "Invalid protocol version");
    }

    #[test]
    fn error_display_crc_mismatch() {
        let err = Error::new(ErrorKind::CrcMismatch);
        assert_eq!(format!("{}", err), "CRC checksum mismatch");
    }

    #[test]
    fn error_display_unexpected_eof() {
        let err = Error::new(ErrorKind::UnexpectedEof);
        assert_eq!(format!("{}", err), "Unexpected end of file");
    }

    #[test]
    fn error_display_invalid_packet() {
        let err = Error::new(ErrorKind::InvalidPacket);
        assert_eq!(format!("{}", err), "Invalid packet");
    }

    #[test]
    fn error_display_write_zero() {
        let err = Error::new(ErrorKind::WriteZero);
        assert_eq!(format!("{}", err), "Write zero bytes");
    }

    #[test]
    fn error_display_interrupted() {
        let err = Error::new(ErrorKind::Interrupted);
        assert_eq!(format!("{}", err), "Operation interrupted");
    }

    #[test]
    fn error_display_other() {
        let err = Error::new(ErrorKind::Other);
        assert_eq!(format!("{}", err), "Other error");
    }

    #[test]
    fn error_debug_format() {
        let err = Error::new(ErrorKind::CrcMismatch);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("CrcMismatch"));
    }

    #[test]
    fn error_kind_clone() {
        let kind = ErrorKind::InvalidMagic;
        let cloned = kind;
        assert_eq!(kind, cloned);
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_to_io_error_unexpected_eof() {
        let err = Error::new(ErrorKind::UnexpectedEof);
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_to_io_error_write_zero() {
        let err = Error::new(ErrorKind::WriteZero);
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::WriteZero);
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_to_io_error_interrupted() {
        let err = Error::new(ErrorKind::Interrupted);
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::Interrupted);
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_to_io_error_other_kinds() {
        let other_kinds = [
            ErrorKind::InvalidMagic,
            ErrorKind::InvalidVersion,
            ErrorKind::CrcMismatch,
            ErrorKind::InvalidPacket,
            ErrorKind::Other,
        ];
        for kind in other_kinds {
            let err = Error::new(kind);
            let io_err: std::io::Error = err.into();
            assert_eq!(io_err.kind(), std::io::ErrorKind::Other);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_implements_std_error() {
        let err = Error::new(ErrorKind::Other);
        let _: &dyn std::error::Error = &err;
    }
}
