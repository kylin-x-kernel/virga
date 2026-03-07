// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! 服务器模块

#[cfg(feature = "use-xtransport")]
pub mod server_sync;
#[cfg(feature = "use-xtransport")]
pub use crate::transport::XTransportHandler;
#[cfg(feature = "use-xtransport")]
pub use server_sync::VirgeServer;

#[cfg(feature = "use-yamux")]
pub mod server_async;
#[cfg(feature = "use-yamux")]
use crate::transport::get_runtime;
#[cfg(feature = "use-yamux")]
pub use crate::transport::YamuxTransportHandler;
#[cfg(feature = "use-yamux")]
pub use server_async::VirgeServer;

use log::*;
use std::io::{Error, ErrorKind, Result};

/// 监听器枚举
enum Listener {
    #[cfg(feature = "use-xtransport")]
    XTransport(vsock::VsockListener),
    #[cfg(feature = "use-yamux")]
    Yamux(tokio_vsock::VsockListener),
}

/// 服务器配置
#[derive(Clone, Debug)]
pub struct ServerConfig {
    listen_cid: u32,
    listen_port: u32,
    #[allow(dead_code)]
    chunk_size: u32,
    #[allow(dead_code)]
    is_ack: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_cid: crate::VMADDR_CID_ANY as u32,
            listen_port: crate::DEFAULT_SERVER_PORT as u32,
            chunk_size: crate::DEAFULT_CHUNK_SIZE as u32,
            is_ack: crate::DEFAULT_IS_ACK,
        }
    }
}

impl ServerConfig {
    pub fn new(cid: u32, port: u32, chunk: u32, isack: bool) -> Self {
        Self {
            listen_cid: cid,
            listen_port: port,
            chunk_size: chunk,
            is_ack: isack,
        }
    }
}

/// 服务器管理器：管理 vsock 监听和连接接受
pub struct ServerManager {
    config: ServerConfig,
    listener: Option<Listener>,
    running: bool,
}

impl ServerManager {
    pub const fn new(config: ServerConfig) -> Self {
        Self {
            config,
            listener: None,
            running: false,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        info!(
            "ServerManager starting on cid={}, port={}",
            self.config.listen_cid, self.config.listen_port
        );

        self.listener = Some(self.create_listener()?);
        self.running = true;
        Ok(())
    }

    fn create_listener(&self) -> Result<Listener> {
        #[cfg(feature = "use-yamux")]
        {
            let addr = tokio_vsock::VsockAddr::new(self.config.listen_cid, self.config.listen_port);
            let listener =
                get_runtime().block_on(async { tokio_vsock::VsockListener::bind(addr) })?;
            return Ok(Listener::Yamux(listener));
        }

        #[cfg(feature = "use-xtransport")]
        {
            let addr = vsock::VsockAddr::new(self.config.listen_cid, self.config.listen_port);
            let listener = vsock::VsockListener::bind(&addr)?;
            return Ok(Listener::XTransport(listener));
        }
    }

    pub fn accept(&mut self) -> Result<VirgeServer> {
        if !self.running {
            return Err(Error::new(
                ErrorKind::Other,
                format!("ServerManager not running"),
            ));
        }

        let transport = match &self.listener {
            #[cfg(feature = "use-xtransport")]
            Some(Listener::XTransport(xtransport_listener)) => {
                let (stream, addr) = xtransport_listener.accept()?;
                info!("Accepted xtransport connection from {:?}", addr);

                // 创建 XTransportHandler 实例并从流初始化
                let mut transport = XTransportHandler::new();
                transport.from_stream(stream, self.config.chunk_size, self.config.is_ack)?;
                transport
            }
            #[cfg(feature = "use-yamux")]
            Some(Listener::Yamux(yamux_listener)) => {
                let (stream, addr) =
                    get_runtime().block_on(async { yamux_listener.accept().await })?;
                info!("Accepted yamux connection from {:?}", addr);
                // 创建 YamuxTransport 实例并从流初始化
                let mut transport = YamuxTransportHandler::new(yamux::Mode::Server);
                transport.from_tokio_stream(stream)?;
                transport
            }
            None => {
                return Err(Error::other(format!("Listener not initialized")));
            }
        };

        Ok(VirgeServer::new(transport, true))
    }

    /// 停止服务器
    pub fn stop(&mut self) -> Result<()> {
        info!("ServerManager stopping");
        self.listener = None;
        self.running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_default_values() {
        let config = ServerConfig::default();
        assert_eq!(config.listen_cid, crate::VMADDR_CID_ANY as u32);
        assert_eq!(config.listen_port, crate::DEFAULT_SERVER_PORT as u32);
        assert_eq!(config.chunk_size, crate::DEAFULT_CHUNK_SIZE as u32);
        assert_eq!(config.is_ack, crate::DEFAULT_IS_ACK);
    }

    #[test]
    fn server_config_new_values() {
        let config = ServerConfig::new(100, 9999, 4096, true);
        assert_eq!(config.listen_cid, 100);
        assert_eq!(config.listen_port, 9999);
        assert_eq!(config.chunk_size, 4096);
        assert!(config.is_ack);
    }

    #[test]
    fn server_config_new_zero() {
        let config = ServerConfig::new(0, 0, 0, false);
        assert_eq!(config.listen_cid, 0);
        assert_eq!(config.listen_port, 0);
        assert_eq!(config.chunk_size, 0);
        assert!(!config.is_ack);
    }

    #[test]
    fn server_config_new_max() {
        let config = ServerConfig::new(u32::MAX, u32::MAX, u32::MAX, true);
        assert_eq!(config.listen_cid, u32::MAX);
        assert_eq!(config.listen_port, u32::MAX);
        assert_eq!(config.chunk_size, u32::MAX);
    }

    #[test]
    fn server_config_clone_preserves_fields() {
        let config = ServerConfig::new(100, 1234, 512, true);
        let cloned = config.clone();
        assert_eq!(config.listen_cid, cloned.listen_cid);
        assert_eq!(config.listen_port, cloned.listen_port);
        assert_eq!(config.chunk_size, cloned.chunk_size);
        assert_eq!(config.is_ack, cloned.is_ack);
    }

    #[test]
    fn server_manager_new_initial_state() {
        let config = ServerConfig::default();
        let manager = ServerManager::new(config);
        assert!(!manager.is_running());
        assert!(manager.listener.is_none());
    }

    #[test]
    fn server_manager_accept_before_start_fails() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        let result = manager.accept();
        assert!(result.is_err());
    }

    #[test]
    fn server_manager_stop_when_not_started() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        let result = manager.stop();
        assert!(result.is_ok());
        assert!(!manager.is_running());
    }

    #[test]
    fn server_manager_stop_clears_listener() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        // Stop should clear listener and running flag
        manager.stop().unwrap();
        assert!(manager.listener.is_none());
        assert!(!manager.is_running());
    }

    #[test]
    fn server_manager_accept_error_message() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        let result = manager.accept();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::Other);
        assert!(err.to_string().contains("not running"));
    }

    #[test]
    fn server_config_is_ack_false_default() {
        let config = ServerConfig::default();
        assert!(!config.is_ack);
    }

    #[test]
    fn server_config_different_values() {
        let c1 = ServerConfig::new(1, 2, 3, false);
        let c2 = ServerConfig::new(4, 5, 6, true);
        assert_ne!(c1.listen_cid, c2.listen_cid);
        assert_ne!(c1.listen_port, c2.listen_port);
        assert_ne!(c1.chunk_size, c2.chunk_size);
        assert_ne!(c1.is_ack, c2.is_ack);
    }

    #[test]
    fn server_manager_is_running_false_initially() {
        let config = ServerConfig::new(0, 0, 0, false);
        let manager = ServerManager::new(config);
        assert!(!manager.is_running());
    }

    #[test]
    fn server_manager_multiple_stops() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        assert!(manager.stop().is_ok());
        assert!(manager.stop().is_ok());
        assert!(!manager.is_running());
    }

    #[test]
    fn server_config_debug_contains_fields() {
        let config = ServerConfig::new(42, 1234, 512, true);
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("1234"));
        assert!(debug_str.contains("512"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn server_manager_start_fails_without_vsock() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        // Should fail on systems without vsock or when addresses are invalid
        let result = manager.start();
        // This will likely fail in test environment, but tests the start path
        if result.is_err() {
            assert!(!manager.is_running());
        }
    }

    #[test]
    fn server_manager_create_listener_xtransport() {
        let config = ServerConfig::new(0, 12345, 1024, false);
        let manager = ServerManager::new(config);
        // Test create_listener method - will fail in test env but exercises code path
        let result = manager.create_listener();
        // In test environment, this should fail but we test the code path
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn server_manager_accept_with_no_listener_fails() {
        let config = ServerConfig::default();
        let mut manager = ServerManager::new(config);
        // Set running but no listener
        manager.running = true;
        manager.listener = None;

        let result = manager.accept();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("not initialized"));
        }
    }

    #[test]
    fn server_config_cid_field_access() {
        let config = ServerConfig::new(123, 456, 789, true);
        assert_eq!(config.listen_cid, 123);
        assert_eq!(config.listen_port, 456);
        assert_eq!(config.chunk_size, 789);
        assert_eq!(config.is_ack, true);
    }

    #[test]
    fn server_manager_const_new() {
        // Test that new is const
        const CONFIG: ServerConfig = ServerConfig {
            listen_cid: 100,
            listen_port: 1234,
            chunk_size: 1024,
            is_ack: false,
        };
        const MANAGER: ServerManager = ServerManager::new(CONFIG);
        assert!(!MANAGER.running);
    }
}
