//! XTransport 传输协议实现
//!
//! 基于 xtransport 库的传输实现。
//!
//! # 特点
//! - 针对 vsock 优化的传输协议
//! - 轻量级设计
//!
//! # 结构
//! ```text
//! ┌─────────────────────────────────┐
//! │ XTransportHandler               │
//! │ - stream: Option<VsockStream>   │
//! │ - transport: Option<XTransport> │
//! └─────────────────────────────────┘
//! ```

use crate::error::{Result, VirgeError};
use crate::transport::Transport;
use std::pin::Pin;
use std::future::Future;

/// XTransport 传输协议实现
///
/// 直接管理 vsock 连接并使用 xtransport 进行传输。
pub struct XTransportHandler {
    /// vsock 连接流
    stream: Option<vsock::VsockStream>,

    /// xtransport 处理器
    transport: Option<xtransport::XTransport<vsock::VsockStream>>,
}

impl XTransportHandler {
    /// 创建新的 XTransport 传输实例
    pub fn new() -> Self {
        Self {
            stream: None,
            transport: None,
        }
    }
}

impl Transport for XTransportHandler {
    fn connect(&mut self, cid: u32, port: u32) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            log::info!("XTransport connecting to cid={}, port={}", cid, port);

            // 建立 vsock 连接
            let addr = vsock::VsockAddr::new(cid, port);
            let stream = vsock::VsockStream::connect(&addr)
                .map_err(|e| VirgeError::ConnectionError(format!("Failed to connect vsock: {}", e)))?;

            // 初始化 xtransport
            let config = xtransport::TransportConfig::default()
                .with_max_frame_size(1024)
                .with_ack(false);
            let transport = xtransport::XTransport::new(stream.try_clone()?, config);

            self.stream = Some(stream);
            self.transport = Some(transport);

            log::info!("XTransport connected successfully");
            Ok(())
        })
    }

    fn from_stream(&mut self, stream: vsock::VsockStream) -> Result<()> {
        log::info!("XTransport initializing from existing stream");

        // 初始化 xtransport
        let config = xtransport::TransportConfig::default()
            .with_max_frame_size(1024)
            .with_ack(false);
        let transport = xtransport::XTransport::new(stream.try_clone()?, config);

        self.stream = Some(stream);
        self.transport = Some(transport);

        log::info!("XTransport initialized from stream successfully");
        Ok(())
    }

    fn disconnect(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            log::info!("XTransport disconnecting");

            // 清理资源
            self.transport = None;
            self.stream = None;

            log::info!("XTransport disconnected");
            Ok(())
        })
    }

    fn send(&mut self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let transport = self.transport.as_mut()
                .ok_or_else(|| VirgeError::TransportError("XTransport not connected".to_string()))?;

            transport.send_message(&data)
                .map_err(|e| VirgeError::Other(format!("XTransport send error: {}", e)))?;

            log::debug!("XTransport sent {} bytes", data.len());
            Ok(())
        })
    }

    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            let transport = self.transport.as_mut()
                .ok_or_else(|| VirgeError::TransportError("XTransport not connected".to_string()))?;

            let data = transport.recv_message()
                .map_err(|e| VirgeError::Other(format!("XTransport recv error: {}", e)))?;

            log::debug!("XTransport received {} bytes", data.len());
            Ok(data)
        })
    }

    fn is_connected(&self) -> bool {
        self.stream.is_some() && self.transport.is_some()
    }
}
