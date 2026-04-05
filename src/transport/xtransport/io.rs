// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

use crate::transport::xtransport::{Error, Result};

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            let n = self.read(buf)?;
            if n == 0 {
                break;
            }
            let tmp = buf;
            buf = &mut tmp[n..];
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(Error::new(
                crate::transport::xtransport::error::ErrorKind::UnexpectedEof,
            ))
        }
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            let n = self.write(buf)?;
            if n == 0 {
                return Err(Error::new(
                    crate::transport::xtransport::error::ErrorKind::WriteZero,
                ));
            }
            buf = &buf[n..];
        }
        Ok(())
    }
}

// Blanket implementations for std types that implement std::io::{Read, Write}
impl<T: std::io::Read> Read for T {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        std::io::Read::read(self, buf).map_err(|e| {
            Error::new(match e.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    crate::transport::xtransport::error::ErrorKind::UnexpectedEof
                }
                std::io::ErrorKind::Interrupted => {
                    crate::transport::xtransport::error::ErrorKind::Interrupted
                }
                _ => crate::transport::xtransport::error::ErrorKind::Other,
            })
        })
    }
}

impl<T: std::io::Write> Write for T {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        std::io::Write::write(self, buf).map_err(|e| {
            Error::new(match e.kind() {
                std::io::ErrorKind::WriteZero => {
                    crate::transport::xtransport::error::ErrorKind::WriteZero
                }
                std::io::ErrorKind::Interrupted => {
                    crate::transport::xtransport::error::ErrorKind::Interrupted
                }
                _ => crate::transport::xtransport::error::ErrorKind::Other,
            })
        })
    }

    fn flush(&mut self) -> Result<()> {
        std::io::Write::flush(self)
            .map_err(|_| Error::new(crate::transport::xtransport::error::ErrorKind::Other))
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::xtransport::error::ErrorKind;
    use crate::transport::xtransport::io::{Read, Write};
    use std::io::Cursor;

    #[test]
    fn cursor_read_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 3];
        let n = cursor.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&buf, &[1, 2, 3]);
    }

    #[test]
    fn cursor_read_exact_success() {
        let data = vec![10, 20, 30, 40, 50];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 5];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, &[10, 20, 30, 40, 50]);
    }

    #[test]
    fn cursor_read_exact_insufficient_data() {
        let data = vec![1, 2, 3];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 5];
        let err = cursor.read_exact(&mut buf).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn cursor_read_exact_empty() {
        let data = vec![1, 2, 3];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 0];
        cursor.read_exact(&mut buf).unwrap();
    }

    #[test]
    fn vec_write_basic() {
        let mut buf: Vec<u8> = Vec::new();
        let n = buf.write(&[1, 2, 3]).unwrap();
        assert_eq!(n, 3);
        assert_eq!(buf, vec![1, 2, 3]);
    }

    #[test]
    fn vec_write_all_success() {
        let mut buf: Vec<u8> = Vec::new();
        buf.write_all(&[10, 20, 30, 40]).unwrap();
        assert_eq!(buf, vec![10, 20, 30, 40]);
    }

    #[test]
    fn vec_flush() {
        let mut buf: Vec<u8> = Vec::new();
        buf.flush().unwrap();
    }

    #[test]
    fn cursor_read_sequential() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 3];

        let n = cursor.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&buf, &[1, 2, 3]);

        let n = cursor.read(&mut buf).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&buf, &[4, 5, 6]);
    }

    #[test]
    fn cursor_read_at_eof() {
        let data = vec![];
        let mut cursor = Cursor::new(&data[..]);
        let mut buf = [0u8; 5];
        let n = cursor.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn vec_write_multiple() {
        let mut buf: Vec<u8> = Vec::new();
        buf.write_all(&[1, 2]).unwrap();
        buf.write_all(&[3, 4]).unwrap();
        assert_eq!(buf, vec![1, 2, 3, 4]);
    }

    #[test]
    fn write_all_empty() {
        let mut buf: Vec<u8> = Vec::new();
        buf.write_all(&[]).unwrap();
        assert!(buf.is_empty());
    }

    // Test write_all with a writer that returns zero bytes (triggers WriteZero error)
    struct ZeroWriter;

    impl std::io::Write for ZeroWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Ok(0) // Always writes zero bytes
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_all_zero_writer_error() {
        let mut writer = ZeroWriter;
        let err = writer.write_all(&[1, 2, 3]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::WriteZero);
    }

    // Test Write blanket impl error mapping for WriteZero and Interrupted
    struct WriteZeroErrorWriter;

    impl std::io::Write for WriteZeroErrorWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "write zero",
            ))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_maps_write_zero_error() {
        let mut writer = WriteZeroErrorWriter;
        let err = Write::write(&mut writer, &[1]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::WriteZero);
    }

    struct InterruptedWriteErrorWriter;

    impl std::io::Write for InterruptedWriteErrorWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "interrupted",
            ))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_maps_interrupted_error() {
        let mut writer = InterruptedWriteErrorWriter;
        let err = Write::write(&mut writer, &[1]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Interrupted);
    }

    // Test flush error mapping
    struct FlushErrorWriter;

    impl std::io::Write for FlushErrorWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "flush error",
            ))
        }
    }

    #[test]
    fn flush_error_mapped() {
        let mut writer = FlushErrorWriter;
        let err = Write::flush(&mut writer).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
    }
}
