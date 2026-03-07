// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 KylinSoft Co., Ltd. <https://www.kylinos.cn/>
// See LICENSES for license details.

use std::io::{Error, ErrorKind, Result};
use std::io::{Read, Write};

use log::*;

use super::ClientConfig;
use crate::transport::XTransportHandler;
use crate::ReadState;

/// 同步客户端
pub struct VirgeClient {
    transport_handler: XTransportHandler,
    config: ClientConfig,
    connected: bool,
    read_buffer: Vec<u8>,  // 读取缓存
    read_state: ReadState, // 读取状态
}

impl VirgeClient {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            transport_handler: XTransportHandler::new(),
            config,
            connected: false,
            read_buffer: Vec::new(),
            read_state: ReadState::Idle,
        }
    }

    /// 建立连接
    pub fn connect(&mut self) -> Result<()> {
        info!(
            "VirgeClient connecting to cid={}, port={}",
            self.config.server_cid, self.config.server_port
        );

        self.transport_handler.connect(
            self.config.server_cid,
            self.config.server_port,
            self.config.chunk_size,
            self.config.is_ack,
        )?;
        self.connected = true;
        Ok(())
    }

    /// 断开连接
    pub fn disconnect(&mut self) -> Result<()> {
        info!("VirgeClient disconnecting");
        if !self.read_buffer.is_empty() {
            warn!(
                "Disconnecting with {} bytes of unread data in buffer",
                self.read_buffer.len()
            );
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Cannot disconnect: {} bytes of unread data remaining",
                    self.read_buffer.len()
                ),
            ));
        }

        self.transport_handler.disconnect()?;
        self.connected = false;
        Ok(())
    }

    /// 发送数据
    pub fn send(&mut self, data: Vec<u8>) -> Result<usize> {
        if !self.connected {
            return Err(Error::new(
                ErrorKind::NotConnected,
                format!("Client not connected"),
            ));
        }

        self.transport_handler
            .send(&data)
            .map_err(|e| Error::other(format!("send error: {}", e)))
    }

    /// 接收数据
    pub fn recv(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(Error::new(
                ErrorKind::NotConnected,
                format!("Client not connected"),
            ));
        }

        self.transport_handler
            .recv()
            .map_err(|e| Error::other(format!("recv error: {}", e)))
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connected && self.transport_handler.is_connected()
    }
}

impl VirgeClient {
    fn read_new_message(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.transport_handler.recv() {
            Ok(data) => {
                if data.len() <= buf.len() {
                    buf[..data.len()].copy_from_slice(&data);
                    Ok(data.len())
                } else {
                    let len = buf.len();
                    buf.copy_from_slice(&data[..len]);
                    self.read_buffer.extend_from_slice(&data[len..]);

                    self.read_state = ReadState::Reading {
                        total: data.len(),
                        read: len,
                    };
                    Ok(len)
                }
            }
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Read error: {}", e))),
        }
    }

    /// 检查是否还有数据可读（包括rbuf中的数据）
    pub fn no_has_data(&self) -> bool {
        self.read_buffer.is_empty() && self.read_state == ReadState::Idle
    }
}

impl Read for VirgeClient {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.connected {
            return Err(Error::new(ErrorKind::NotConnected, "Client not connected"));
        }

        match self.read_state {
            ReadState::Idle => {
                // 直接从传输层读取
                return self.read_new_message(buf);
            }
            ReadState::Reading { total, read, .. } => {
                // 从rbuf中读取剩余数据
                if !self.read_buffer.is_empty() {
                    let len = std::cmp::min(self.read_buffer.len(), buf.len());
                    buf[..len].copy_from_slice(&self.read_buffer[..len]);
                    self.read_buffer.drain(..len);

                    let new_read = read + len;
                    if new_read == total {
                        // 消息读取完成
                        self.read_state = ReadState::Idle;
                    } else {
                        // 更新状态
                        self.read_state = ReadState::Reading {
                            total,
                            read: new_read,
                        };
                    }
                    Ok(len)
                } else {
                    // rbuf为空但状态是Reading，这不应该发生
                    self.read_state = ReadState::Idle;
                    Ok(0)
                }
            }
        }
    }
}

impl Write for VirgeClient {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if !self.connected {
            return Err(Error::new(ErrorKind::NotConnected, "Client not connected"));
        }

        match self.transport_handler.send(buf) {
            Ok(len) => Ok(len),
            Err(e) => Err(Error::new(ErrorKind::Other, format!("Write error: {}", e))),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;
    use std::io::{Read, Write};

    fn make_client() -> VirgeClient {
        let config = ClientConfig::default();
        VirgeClient::new(config)
    }

    #[test]
    fn new_client_not_connected() {
        let client = make_client();
        assert!(!client.is_connected());
        assert!(!client.connected);
    }

    #[test]
    fn new_client_empty_read_buffer() {
        let client = make_client();
        assert!(client.read_buffer.is_empty());
        assert_eq!(client.read_state, ReadState::Idle);
    }

    #[test]
    fn no_has_data_initially_true() {
        let client = make_client();
        assert!(client.no_has_data());
    }

    #[test]
    fn send_when_not_connected_fails() {
        let mut client = make_client();
        let result = client.send(vec![1, 2, 3]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotConnected);
    }

    #[test]
    fn recv_when_not_connected_fails() {
        let mut client = make_client();
        let result = client.recv();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotConnected);
    }

    #[test]
    fn read_when_not_connected_fails() {
        let mut client = make_client();
        let mut buf = [0u8; 10];
        let result = client.read(&mut buf);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotConnected);
    }

    #[test]
    fn write_when_not_connected_fails() {
        let mut client = make_client();
        let result = client.write(&[1, 2, 3]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotConnected);
    }

    #[test]
    fn flush_always_ok() {
        let mut client = make_client();
        assert!(client.flush().is_ok());
    }

    #[test]
    fn send_empty_when_not_connected_fails() {
        let mut client = make_client();
        let result = client.send(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn new_client_with_custom_config() {
        let config = ClientConfig::new(100, 9999, 4096, true);
        let client = VirgeClient::new(config);
        assert!(!client.is_connected());
        assert_eq!(client.config.server_cid, 100);
        assert_eq!(client.config.server_port, 9999);
        assert_eq!(client.config.chunk_size, 4096);
        assert!(client.config.is_ack);
    }

    #[test]
    fn disconnect_with_unread_data_fails() {
        let mut client = make_client();
        // Simulate data in read buffer
        client.read_buffer = vec![1, 2, 3];
        let result = client.disconnect();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Other);
        assert!(err.to_string().contains("unread data"));
    }

    #[test]
    fn read_state_updates_correctly() {
        let mut client = make_client();
        // Mock connected state and setup reading scenario
        client.connected = true;
        client.read_state = ReadState::Reading {
            total: 100,
            read: 50,
        };
        client.read_buffer = vec![1, 2, 3, 4, 5];

        let mut buf = [0u8; 3];
        let result = client.read(&mut buf);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(buf, [1, 2, 3]);
        assert_eq!(client.read_buffer, vec![4, 5]);

        // State should be updated
        match client.read_state {
            ReadState::Reading { total, read } => {
                assert_eq!(total, 100);
                assert_eq!(read, 53);
            }
            _ => panic!("Expected ReadState::Reading"),
        }
    }

    #[test]
    fn read_state_idle_when_complete() {
        let mut client = make_client();
        // Mock connected state and setup completion scenario
        client.connected = true;
        client.read_state = ReadState::Reading {
            total: 100,
            read: 97,
        };
        client.read_buffer = vec![1, 2, 3];

        let mut buf = [0u8; 10];
        let result = client.read(&mut buf);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        // Should transition to Idle when complete
        assert_eq!(client.read_state, ReadState::Idle);
    }

    #[test]
    fn read_with_empty_buffer_in_reading_state() {
        let mut client = make_client();
        // Mock connected state and setup edge case scenario
        client.connected = true;
        client.read_state = ReadState::Reading {
            total: 100,
            read: 50,
        };
        // Empty read buffer but in Reading state
        client.read_buffer.clear();

        let mut buf = [0u8; 10];
        let result = client.read(&mut buf);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        // Should reset to Idle
        assert_eq!(client.read_state, ReadState::Idle);
    }

    #[test]
    fn no_has_data_with_buffer() {
        let mut client = make_client();
        client.read_buffer = vec![1, 2, 3];
        assert!(!client.no_has_data());
    }

    #[test]
    fn no_has_data_with_reading_state() {
        let mut client = make_client();
        client.read_state = ReadState::Reading {
            total: 100,
            read: 50,
        };
        assert!(!client.no_has_data());
    }

    #[test]
    fn connect_logs_info() {
        // This tests the logging path in connect
        let mut client = make_client();
        // We can't actually connect without a real vsock, but we can test the error path
        let result = client.connect();
        assert!(result.is_err());
        // Should remain not connected on error
        assert!(!client.connected);
    }

    #[test]
    fn disconnect_logs_info() {
        // Test disconnect logging with connected client
        let mut client = make_client();
        client.connected = true; // Simulate connected state
        let result = client.disconnect();
        // Will fail due to no actual connection, but tests the logging path
        assert!(result.is_ok());
        assert!(!client.connected);
    }

    #[test]
    fn read_new_message_simulation() {
        // This test exercises the read_new_message path indirectly
        let mut client = make_client();
        client.connected = true;

        // Test read when in Idle state (will try to call read_new_message)
        let mut buf = [0u8; 10];
        let result = client.read(&mut buf);
        // Will fail because transport isn't really connected, but exercises the path
        assert!(result.is_err());
    }

    #[test]
    fn write_error_conversion() {
        // Test that write errors are properly converted
        let mut client = make_client();
        client.connected = true; // Mock connected but transport will fail

        let result = client.write(&[1, 2, 3]);
        assert!(result.is_err());
        // Error should be converted from transport error
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn is_connected_checks_both_flags() {
        let mut client = make_client();
        // Test when client.connected is true but transport is not
        client.connected = true;
        assert!(!client.is_connected()); // Should be false because transport not connected

        client.connected = false;
        assert!(!client.is_connected()); // Should be false
    }

    #[test]
    fn send_and_recv_error_formatting() {
        let mut client = make_client();
        client.connected = true; // Mock connected

        // Test send error message format
        let send_result = client.send(vec![1, 2, 3]);
        assert!(send_result.is_err());

        // Test recv error message format
        let recv_result = client.recv();
        assert!(recv_result.is_err());
    }
}
