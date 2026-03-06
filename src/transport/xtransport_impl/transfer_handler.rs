// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

//! XTransport 传输协议实现
//!
//! 基于 xtransport 库的传输实现。
//!
//! # 特点
//! - 针对 vsock 优化的传输协议
//! - 轻量级设计

use crate::error::{Result, VirgeError};
use log::*;
use vsock::{VsockAddr, VsockStream};
use xtransport::{TransportConfig, XTransport};

/// XTransport 传输协议实现
///
/// 直接管理 vsock 连接并使用 xtransport 进行传输。
pub struct XTransportHandler {
    stream: Option<VsockStream>,
    transport: Option<XTransport<VsockStream>>,
}

impl XTransportHandler {
    pub fn new() -> Self {
        Self {
            stream: None,
            transport: None,
        }
    }
}

impl XTransportHandler {
    pub fn connect(&mut self, cid: u32, port: u32, chunksize: u32, isack: bool) -> Result<()> {
        debug!("XTransport connecting to cid={}, port={}", cid, port);

        let stream = VsockStream::connect(&VsockAddr::new(cid, port))
            .map_err(|e| VirgeError::ConnectionError(format!("Failed to connect vsock: {}", e)))?;

        let config = TransportConfig::default()
            .with_max_frame_size(chunksize as usize)
            .with_ack(isack);
        let transport = XTransport::new(stream.try_clone()?, config);

        self.stream = Some(stream);
        self.transport = Some(transport);

        debug!("XTransport connected successfully");
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        debug!("XTransport disconnecting");

        self.transport = None;
        if let Some(stream) = &self.stream {
            stream.shutdown(std::net::Shutdown::Both).map_err(|e| {
                VirgeError::ConnectionError(format!("Failed to disconnect vsock: {}", e))
            })?;
        }

        debug!("XTransport disconnected");
        Ok(())
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| VirgeError::TransportError("XTransport not connected".to_string()))?;

        transport
            .send_message(data)
            .map_err(|e| VirgeError::Other(format!("XTransport send error: {}", e)))?;

        debug!("XTransport sent {} bytes", data.len());
        Ok(data.len())
    }

    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| VirgeError::TransportError("XTransport not connected".to_string()))?;

        let data = transport
            .recv_message()
            .map_err(|e| VirgeError::Other(format!("XTransport recv error: {}", e)))?;

        debug!("XTransport received {} bytes", data.len());
        Ok(data)
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.transport.is_some()
    }

    pub fn from_stream(&mut self, stream: VsockStream, chunksize: u32, isack: bool) -> Result<()> {
        debug!("XTransport initializing from existing stream");

        let config = TransportConfig::default()
            .with_max_frame_size(chunksize as usize)
            .with_ack(isack);
        let transport = XTransport::new(stream.try_clone()?, config);

        self.stream = Some(stream);
        self.transport = Some(transport);

        debug!("XTransport initialized from stream successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_handler_not_connected() {
        let handler = XTransportHandler::new();
        assert!(!handler.is_connected());
        assert!(handler.stream.is_none());
        assert!(handler.transport.is_none());
    }

    #[test]
    fn send_without_connection_fails() {
        let mut handler = XTransportHandler::new();
        let result = handler.send(&[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn recv_without_connection_fails() {
        let mut handler = XTransportHandler::new();
        let result = handler.recv();
        assert!(result.is_err());
    }

    #[test]
    fn disconnect_without_connection_ok() {
        let mut handler = XTransportHandler::new();
        // No stream to shutdown, transport is None
        let result = handler.disconnect();
        assert!(result.is_ok());
    }

    #[test]
    fn is_connected_false_when_no_stream() {
        let handler = XTransportHandler::new();
        assert!(!handler.is_connected());
    }

    #[test]
    fn send_empty_without_connection_fails() {
        let mut handler = XTransportHandler::new();
        let result = handler.send(&[]);
        assert!(result.is_err());
    }
}
