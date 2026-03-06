// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! 客户端模块

#[cfg(feature = "use-xtransport")]
pub mod client_sync;
#[cfg(feature = "use-xtransport")]
pub use client_sync::VirgeClient;

#[cfg(feature = "use-yamux")]
pub mod client_async;
#[cfg(feature = "use-yamux")]
pub use client_async::VirgeClient;

/// 客户端配置
#[derive(Clone, Debug)]
pub struct ClientConfig {
    server_cid: u32,
    server_port: u32,
    chunk_size: u32,
    is_ack: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_cid: crate::DEFAULT_SERVER_CID as u32,
            server_port: crate::DEFAULT_SERVER_PORT as u32,
            chunk_size: crate::DEAFULT_CHUNK_SIZE as u32,
            is_ack: crate::DEFAULT_IS_ACK,
        }
    }
}

impl ClientConfig {
    pub fn new(cid: u32, port: u32, chunk: u32, isack: bool) -> Self {
        Self {
            server_cid: cid,
            server_port: port,
            chunk_size: chunk,
            is_ack: isack,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_config_default_values() {
        let config = ClientConfig::default();
        assert_eq!(config.server_cid, crate::DEFAULT_SERVER_CID as u32);
        assert_eq!(config.server_port, crate::DEFAULT_SERVER_PORT as u32);
        assert_eq!(config.chunk_size, crate::DEAFULT_CHUNK_SIZE as u32);
        assert_eq!(config.is_ack, crate::DEFAULT_IS_ACK);
    }

    #[test]
    fn client_config_new_values() {
        let config = ClientConfig::new(200, 5678, 2048, true);
        assert_eq!(config.server_cid, 200);
        assert_eq!(config.server_port, 5678);
        assert_eq!(config.chunk_size, 2048);
        assert!(config.is_ack);
    }

    #[test]
    fn client_config_new_zero() {
        let config = ClientConfig::new(0, 0, 0, false);
        assert_eq!(config.server_cid, 0);
        assert_eq!(config.server_port, 0);
        assert_eq!(config.chunk_size, 0);
        assert!(!config.is_ack);
    }

    #[test]
    fn client_config_new_max() {
        let config = ClientConfig::new(u32::MAX, u32::MAX, u32::MAX, true);
        assert_eq!(config.server_cid, u32::MAX);
        assert_eq!(config.server_port, u32::MAX);
        assert_eq!(config.chunk_size, u32::MAX);
    }

    #[test]
    fn client_config_clone_preserves_fields() {
        let config = ClientConfig::new(100, 1234, 512, true);
        let cloned = config.clone();
        assert_eq!(config.server_cid, cloned.server_cid);
        assert_eq!(config.server_port, cloned.server_port);
        assert_eq!(config.chunk_size, cloned.chunk_size);
        assert_eq!(config.is_ack, cloned.is_ack);
    }
}
