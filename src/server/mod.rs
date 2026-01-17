//! 服务器模块
//!
//! 提供服务器角色的高级 API。
//!
//! # 职责
//! - 封装服务器的监听逻辑
//! - 处理客户端连接
//! - 为每个连接创建独立的 Transport 实例
//! - 管理传输协议选择
//!
//! # 设计思路
//! ```text
//! ┌────────────────────────────────────┐
//! │ VirgeServer                        │
//! │ - config: ServerConfig             │
//! │ - listener: Option<VsockListener>  │
//! │ - transport_factory: TransportType │
//! └────────────────────────────────────┘
//!          │
//!          ├─ listen()
//!          ├─ accept() -> Box<dyn Transport>
//!          └─ stop()
//! ```
//!
//! # 使用示例
//! ```ignore
//! let mut server = VirgeServer::with_yamux(config);
//! server.listen().await?;
//!
//! while let Ok(mut transport) = server.accept().await {
//!     tokio::spawn(async move {
//!         let data = transport.recv().await?;
//!         transport.send(data).await?;
//!         Ok::<(), Box<dyn std::error::Error>>(())
//!     });
//! }
//! ```

use crate::error::Result;
use crate::transport::Transport;

/// 传输协议类型
#[derive(Clone, Debug)]
pub enum TransportType {
    #[cfg(feature = "use-yamux")]
    Yamux,
    #[cfg(feature = "use-xtransport")]
    XTransport,
}

/// 服务器配置
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// 服务器监听的 CID
    pub listen_cid: u32,

    /// 服务器监听的端口
    pub listen_port: u32,

    /// 最大并发连接数
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_cid: crate::DEFAULT_SERVER_CID as u32,
            listen_port: crate::DEFAULT_SERVER_PORT as u32,
            max_connections: 100,
        }
    }
}


/// Virga 服务器
///
/// 提供基于选定传输协议的高级服务器接口。
pub struct VirgeServer {
    /// 服务器配置
    config: ServerConfig,

    /// 传输协议类型
    transport_type: TransportType,

    /// vsock 监听器
    #[cfg(feature = "use-yamux")]
    yamux_listener: Option<tokio_vsock::VsockListener>,

    #[cfg(feature = "use-xtransport")]
    xtransport_listener: Option<vsock::VsockListener>,

    /// 监听状态
    listening: bool,
}

impl VirgeServer {
    /// 使用 Yamux 创建服务器
    #[cfg(feature = "use-yamux")]
    pub fn with_yamux(config: ServerConfig) -> Self {
        Self {
            config,
            transport_type: TransportType::Yamux,
            #[cfg(feature = "use-yamux")]
            yamux_listener: None,
            #[cfg(feature = "use-xtransport")]
            xtransport_listener: None,
            listening: false,
        }
    }

    /// 使用 XTransport 创建服务器
    #[cfg(feature = "use-xtransport")]
    pub fn with_xtransport(config: ServerConfig) -> Self {
        Self {
            config,
            transport_type: TransportType::XTransport,
            #[cfg(feature = "use-yamux")]
            yamux_listener: None,
            #[cfg(feature = "use-xtransport")]
            xtransport_listener: None,
            listening: false,
        }
    }

    /// 启动监听
    pub async fn listen(&mut self) -> Result<()> {
        log::info!(
            "VirgeServer listening on cid={}, port={}",
            self.config.listen_cid,
            self.config.listen_port
        );

        match self.transport_type {
            #[cfg(feature = "use-yamux")]
            TransportType::Yamux => {
                let addr = tokio_vsock::VsockAddr::new(self.config.listen_cid, self.config.listen_port);
                let listener = tokio_vsock::VsockListener::bind(addr)
                    .map_err(|e| crate::error::VirgeError::ConnectionError(format!("Failed to bind yamux listener: {}", e)))?;
                self.yamux_listener = Some(listener);
                self.listening = true;
                Ok(())
            }
            #[cfg(feature = "use-xtransport")]
            TransportType::XTransport => {
                let addr = vsock::VsockAddr::new(self.config.listen_cid, self.config.listen_port);
                let listener = vsock::VsockListener::bind(&addr)
                    .map_err(|e| crate::error::VirgeError::ConnectionError(format!("Failed to bind xtransport listener: {}", e)))?;
                self.xtransport_listener = Some(listener);
                self.listening = true;
                Ok(())
            }
            #[cfg(not(any(feature = "use-yamux", feature = "use-xtransport")))]
            TransportType::Yamux | TransportType::XTransport => {
                Err(crate::error::VirgeError::Other("Transport feature not enabled".to_string()))
            }
        }
    }

    /// 接受新的客户端连接
    pub async fn accept(&mut self) -> Result<Box<dyn Transport>> {
        if !self.listening {
            return Err(crate::error::VirgeError::Other(
                "Server not listening".to_string(),
            ));
        }

        match self.transport_type {
            #[cfg(feature = "use-yamux")]
            TransportType::Yamux => {
                if let Some(listener) = &mut self.yamux_listener {
                    let (stream, addr) = listener.accept().await
                        .map_err(|e| crate::error::VirgeError::ConnectionError(format!("Failed to accept yamux connection: {}", e)))?;
                    log::info!("Accepted yamux connection from {:?}", addr);

                    // 创建 YamuxTransport 实例并从流初始化
                    let mut transport = Box::new(crate::transport::YamuxTransport::new());
                    transport.from_tokio_stream(stream).await?;

                    Ok(transport)
                } else {
                    Err(crate::error::VirgeError::Other("Yamux listener not initialized".to_string()))
                }
            }
            #[cfg(feature = "use-xtransport")]
            TransportType::XTransport => {
                if let Some(listener) = &mut self.xtransport_listener {
                    let (stream, addr) = listener.accept()
                        .map_err(|e| crate::error::VirgeError::ConnectionError(format!("Failed to accept xtransport connection: {}", e)))?;
                    log::info!("Accepted xtransport connection from {:?}", addr);

                    // 创建 XTransportHandler 实例并从流初始化
                    let mut transport = Box::new(crate::transport::XTransportHandler::new());
                    transport.from_stream(stream)?;

                    Ok(transport)
                } else {
                    Err(crate::error::VirgeError::Other("XTransport listener not initialized".to_string()))
                }
            }
            #[cfg(not(any(feature = "use-yamux", feature = "use-xtransport")))]
            TransportType::Yamux | TransportType::XTransport => {
                Err(crate::error::VirgeError::Other("Transport feature not enabled".to_string()))
            }
        }
    }

    /// 停止监听
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("VirgeServer stopping");

        // 清理监听器
        #[cfg(feature = "use-yamux")]
        {
            self.yamux_listener = None;
        }
        #[cfg(feature = "use-xtransport")]
        {
            self.xtransport_listener = None;
        }

        self.listening = false;
        Ok(())
    }

    /// 检查监听状态
    pub fn is_listening(&self) -> bool {
        self.listening
    }
}
