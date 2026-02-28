use std::io::{Error, ErrorKind, Result};
use std::io::{Read, Write};

use log::*;

use super::ClientConfig;
use crate::ReadState;
use crate::transport::YamuxTransportHandler;

/// Yamux 客户端（同步接口，内部通过 tokio runtime 驱动 yamux）
pub struct VirgeClient {
    transport_handler: YamuxTransportHandler,
    config: ClientConfig,
    connected: bool,
    read_buffer: Vec<u8>,
    read_state: ReadState,
}

impl VirgeClient {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            transport_handler: YamuxTransportHandler::new(yamux::Mode::Client),
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
            Err(e) => Err(Error::new(ErrorKind::Other, format!("Read error: {}", e))),
        }
    }

    /// 检查是否还有数据可读（包括 read_buffer 中的数据）
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
            ReadState::Idle => self.read_new_message(buf),
            ReadState::Reading { total, read, .. } => {
                if !self.read_buffer.is_empty() {
                    let len = std::cmp::min(self.read_buffer.len(), buf.len());
                    buf[..len].copy_from_slice(&self.read_buffer[..len]);
                    self.read_buffer.drain(..len);

                    let new_read = read + len;
                    if new_read == total {
                        self.read_state = ReadState::Idle;
                    } else {
                        self.read_state = ReadState::Reading {
                            total,
                            read: new_read,
                        };
                    }
                    Ok(len)
                } else {
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
