//! Yamux 传输协议实现
//!
//! 基于 yamux 库的多路复用传输实现。
//!
//! # 特点
//! - 支持多个独立的虚拟流
//! - 适合多并发场景
//! - 由 libp2p 社区维护
//!
//! # 结构
//! ```text
//! ┌─────────────────────────────────┐
//! │ YamuxTransport                  │
//! │ - connection: Option<Connection>│
//! │ - yamux_stream: Option<Stream>  │
//! └─────────────────────────────────┘
//! ```

use crate::error::{Result, VirgeError};
use crate::transport::Transport;
use futures::future::poll_fn;
use futures::AsyncReadExt;
use futures::AsyncWriteExt;
use std::pin::Pin;
use std::future::Future;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_vsock::VsockStream;

/// Yamux 传输协议实现
///
/// 直接管理 tokio-vsock 连接并使用 yamux 进行多路复用。
pub struct YamuxTransport {
    /// yamux 连接
    connection: Option<yamux::Connection<tokio_util::compat::Compat<tokio_vsock::VsockStream>>>,

    /// 当前使用的 yamux 虚拟流
    yamux_stream: Option<yamux::Stream>,
}

impl YamuxTransport {
    /// 创建新的 Yamux 传输实例
    pub fn new() -> Self {
        Self {
            connection: None,
            yamux_stream: None,
        }
    }

    /// 获取或创建 yamux 虚拟流
    async fn get_or_create_stream(&mut self) -> Result<&mut yamux::Stream> {
        if self.yamux_stream.is_none() {
            if let Some(connection) = &mut self.connection {
                // 打开新的虚拟流
                let stream = poll_fn(|cx| connection.poll_new_outbound(cx)).await
                    .map_err(|e| VirgeError::TransportError(format!("Failed to open yamux stream: {}", e)))?;
                self.yamux_stream = Some(stream);
            } else {
                return Err(VirgeError::TransportError("Yamux not initialized".to_string()));
            }
        }

        Ok(self.yamux_stream.as_mut().unwrap())
    }
}

impl Transport for YamuxTransport {
    fn connect(&mut self, cid: u32, port: u32) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            log::info!("Yamux transport connecting to cid={}, port={}", cid, port);

            // 建立 vsock 连接
            let stream = VsockStream::connect(tokio_vsock::VsockAddr::new(cid, port))
                .await
                .map_err(|e| VirgeError::ConnectionError(format!("Failed to connect vsock: {}", e)))?;

            // 初始化 yamux
            let config = yamux::Config::default();
            let connection = yamux::Connection::new(stream.compat(), config, yamux::Mode::Client);

            self.connection = Some(connection);

            log::info!("Yamux transport connected successfully");
            Ok(())
        })
    }

    fn disconnect(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            log::info!("Yamux transport disconnecting");

            // 清理资源
            self.yamux_stream = None;
            self.connection = None;

            log::info!("Yamux transport disconnected");
            Ok(())
        })
    }

    fn send(&mut self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            if !self.is_connected() {
                return Err(VirgeError::TransportError(
                    "Yamux transport not connected".to_string(),
                ));
            }

            let stream = self.get_or_create_stream().await?;
            stream.write_all(&data).await
                .map_err(|e| VirgeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

            log::debug!("Yamux sent {} bytes", data.len());
            Ok(())
        })
    }

    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            if !self.is_connected() {
                return Err(VirgeError::TransportError(
                    "Yamux transport not connected".to_string(),
                ));
            }

            let stream = self.get_or_create_stream().await?;
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await
                .map_err(|e| VirgeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

            log::debug!("Yamux received {} bytes", buf.len());
            Ok(buf)
        })
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

impl YamuxTransport {
    /// 从现有 tokio-vsock 流初始化传输协议（服务器模式）
    pub async fn from_tokio_stream(&mut self, stream: tokio_vsock::VsockStream) -> Result<()> {
        log::info!("Yamux transport initializing from existing tokio stream");

        // 初始化 yamux
        let config = yamux::Config::default();
        let connection = yamux::Connection::new(stream.compat(), config, yamux::Mode::Server);

        self.connection = Some(connection);

        log::info!("Yamux transport initialized from stream successfully");
        Ok(())
    }
}
