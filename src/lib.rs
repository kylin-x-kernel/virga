// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! Virga: 基于 vsock 的传输库
//!
//! 支持 Yamux 和 XTransport 协议，提供同步 API。
//!
//! # 快速开始
//!
//! ```ignore
//! use virga::client::{VirgeClient, ClientConfig};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ClientConfig::new(103, 1234, 1024, false);
//!     let mut client = VirgeClient::new(config);
//!     client.connect()?;
//!     
//!     client.send(vec![1, 2, 3])?;
//!     let data = client.recv()?;
//!     
//!     client.disconnect()?;
//!     Ok(())
//! }
//! ```

#[cfg(all(feature = "use-xtransport", feature = "use-yamux"))]
compile_error!("feature1 and feature2 cannot be enabled at the same time");

pub mod error;
pub use error::{Result, VirgeError};

pub mod client;
pub mod server;
pub mod transport;

pub use client::{ClientConfig, VirgeClient};
pub use server::{ServerConfig, ServerManager, VirgeServer};

pub const KIB: usize = 1024;
pub const MIB: usize = KIB * 1024;
pub const GIB: usize = MIB * 1024;

pub const DEFAULT_SERVER_CID: usize = 103;
pub const VMADDR_CID_ANY: usize = 0xFFFFFFFF;
pub const DEFAULT_SERVER_PORT: usize = 1234;

pub const DEAFULT_CHUNK_SIZE: usize = KIB;
pub const DEFAULT_IS_ACK: bool = false;

#[derive(Debug, PartialEq)]
enum ReadState {
    Idle,
    Reading { total: usize, read: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_kib() {
        assert_eq!(KIB, 1024);
    }

    #[test]
    fn constants_mib() {
        assert_eq!(MIB, 1024 * 1024);
    }

    #[test]
    fn constants_gib() {
        assert_eq!(GIB, 1024 * 1024 * 1024);
    }

    #[test]
    fn constants_default_server_cid() {
        assert_eq!(DEFAULT_SERVER_CID, 103);
    }

    #[test]
    fn constants_vmaddr_cid_any() {
        assert_eq!(VMADDR_CID_ANY, 0xFFFFFFFF);
    }

    #[test]
    fn constants_default_server_port() {
        assert_eq!(DEFAULT_SERVER_PORT, 1234);
    }

    #[test]
    fn constants_default_chunk_size() {
        assert_eq!(DEAFULT_CHUNK_SIZE, KIB);
    }

    #[test]
    fn constants_default_is_ack() {
        assert!(!DEFAULT_IS_ACK);
    }

    #[test]
    fn read_state_idle_eq() {
        assert_eq!(ReadState::Idle, ReadState::Idle);
    }

    #[test]
    fn read_state_reading_eq() {
        let s1 = ReadState::Reading {
            total: 100,
            read: 50,
        };
        let s2 = ReadState::Reading {
            total: 100,
            read: 50,
        };
        assert_eq!(s1, s2);
    }

    #[test]
    fn read_state_different_ne() {
        let idle = ReadState::Idle;
        let reading = ReadState::Reading {
            total: 100,
            read: 0,
        };
        assert_ne!(idle, reading);
    }

    #[test]
    fn read_state_reading_different_values_ne() {
        let s1 = ReadState::Reading {
            total: 100,
            read: 50,
        };
        let s2 = ReadState::Reading {
            total: 100,
            read: 60,
        };
        assert_ne!(s1, s2);
    }

    #[test]
    fn client_config_default() {
        let _config = ClientConfig::default();
        // Default should not panic
    }

    #[test]
    fn client_config_new() {
        let _config = ClientConfig::new(200, 5678, 2048, true);
        // Construction should not panic
    }

    #[test]
    fn client_config_clone() {
        let config = ClientConfig::new(100, 1234, 512, false);
        let _cloned = config.clone();
    }

    #[test]
    fn client_config_debug() {
        let config = ClientConfig::new(103, 1234, 1024, false);
        let debug = format!("{:?}", config);
        assert!(debug.contains("103"));
        assert!(debug.contains("1234"));
    }

    #[test]
    fn server_config_default() {
        let _config = ServerConfig::default();
    }

    #[test]
    fn server_config_new() {
        let _config = ServerConfig::new(100, 9999, 4096, true);
    }

    #[test]
    fn server_config_clone() {
        let config = ServerConfig::new(100, 1234, 512, false);
        let _cloned = config.clone();
    }

    #[test]
    fn server_config_debug() {
        let config = ServerConfig::new(0xFFFFFFFF, 1234, 1024, false);
        let debug = format!("{:?}", config);
        assert!(debug.contains("1234"));
    }

    #[test]
    fn server_manager_new_not_running() {
        let config = ServerConfig::default();
        let manager = ServerManager::new(config);
        assert!(!manager.is_running());
    }

    #[test]
    fn server_manager_accept_before_start() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        let result = manager.accept();
        assert!(result.is_err());
    }
}
