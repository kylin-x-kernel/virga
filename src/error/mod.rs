// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! 错误定义模块
//!
//! 统一定义库中所有的错误类型，使用 `thiserror` 或自定义 enum。
//!
//! # 错误分类
//! - `ConnectionError`：vsock 连接相关错误（连接失败、超时等）
//! - `TransportError`：传输协议相关错误（编码、解码、发送、接收失败）
//! - `InvalidConfig`：配置参数非法
//! - `Unknown`：未知错误

use std::fmt;

/// 库的统一错误类型
#[derive(Debug)]
pub enum VirgeError {
    /// 连接层错误
    ConnectionError(String),

    /// 传输层错误
    TransportError(String),

    /// 配置错误
    ConfigError(String),

    /// IO 错误
    IoError(std::io::Error),

    /// 其他错误
    Other(String),
}

impl fmt::Display for VirgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirgeError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            VirgeError::TransportError(msg) => write!(f, "Transport error: {}", msg),
            VirgeError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            VirgeError::IoError(e) => write!(f, "IO error: {}", e),
            VirgeError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for VirgeError {}

impl From<std::io::Error> for VirgeError {
    fn from(err: std::io::Error) -> Self {
        VirgeError::IoError(err)
    }
}

impl From<VirgeError> for std::io::Error {
    fn from(err: VirgeError) -> Self {
        match err {
            VirgeError::IoError(e) => e,
            VirgeError::ConnectionError(msg) => {
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, msg)
            }
            VirgeError::TransportError(msg) => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
            }
            VirgeError::ConfigError(msg) => {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, msg)
            }
            VirgeError::Other(msg) => std::io::Error::new(std::io::ErrorKind::Other, msg),
        }
    }
}

#[cfg(feature = "use-xtransport")]
impl From<crate::transport::xtransport::Error> for VirgeError {
    fn from(err: crate::transport::xtransport::Error) -> Self {
        VirgeError::Other(format!("XTransport error: {}", err))
    }
}

/// 操作结果类型别名
pub type Result<T> = std::result::Result<T, VirgeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_connection_error() {
        let err = VirgeError::ConnectionError("timeout".to_string());
        assert_eq!(format!("{}", err), "Connection error: timeout");
    }

    #[test]
    fn display_transport_error() {
        let err = VirgeError::TransportError("decode failed".to_string());
        assert_eq!(format!("{}", err), "Transport error: decode failed");
    }

    #[test]
    fn display_config_error() {
        let err = VirgeError::ConfigError("invalid port".to_string());
        assert_eq!(format!("{}", err), "Config error: invalid port");
    }

    #[test]
    fn display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        let err = VirgeError::IoError(io_err);
        assert!(format!("{}", err).contains("IO error:"));
        assert!(format!("{}", err).contains("pipe broken"));
    }

    #[test]
    fn display_other_error() {
        let err = VirgeError::Other("something".to_string());
        assert_eq!(format!("{}", err), "Error: something");
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let virge_err: VirgeError = io_err.into();
        match virge_err {
            VirgeError::IoError(_) => {} // expected
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn into_io_error_from_io_error() {
        let original = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let kind = original.kind();
        let virge_err = VirgeError::IoError(original);
        let io_err: std::io::Error = virge_err.into();
        assert_eq!(io_err.kind(), kind);
    }

    #[test]
    fn into_io_error_connection_error() {
        let err = VirgeError::ConnectionError("refused".to_string());
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::ConnectionRefused);
    }

    #[test]
    fn into_io_error_transport_error() {
        let err = VirgeError::TransportError("bad data".to_string());
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn into_io_error_config_error() {
        let err = VirgeError::ConfigError("invalid".to_string());
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn into_io_error_other() {
        let err = VirgeError::Other("misc".to_string());
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn error_debug_format() {
        let err = VirgeError::ConnectionError("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ConnectionError"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn error_implements_std_error() {
        let err = VirgeError::Other("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[cfg(feature = "use-xtransport")]
    #[test]
    fn from_xtransport_error() {
        let xt_err = crate::transport::xtransport::Error::new(
            crate::transport::xtransport::error::ErrorKind::CrcMismatch,
        );
        let virge_err: VirgeError = xt_err.into();
        match virge_err {
            VirgeError::Other(msg) => {
                assert!(msg.contains("XTransport error"));
            }
            _ => panic!("Expected Other variant"),
        }
    }
}
