use std::io::{Read, Write};
use std::io::{Error, ErrorKind, Result};

use log::*;

use crate::ReadState;
use crate::transport::YamuxTransportHandler;

/// Virga 服务器连接：与VirgeClient类似，负责单个连接的数据传输。
pub struct VirgeServer {
    transport_handler: YamuxTransportHandler,
    connected: bool,
    read_buffer: Vec<u8>,
    read_state: ReadState,
}

impl VirgeServer {
    pub fn new(trans: YamuxTransportHandler, conn: bool) -> Self {
        Self {
            transport_handler: trans,
            connected: conn,
            read_buffer: Vec::new(),
            read_state: ReadState::Idle,
        }
    }
}

impl VirgeServer {
    /// 发送数据
    pub fn send(&mut self, data: Vec<u8>) -> Result<usize> {
        if !self.connected {
            return Err(Error::new(
                ErrorKind::NotConnected,
                "Server not connected",
            ));
        }
        self.transport_handler.send(&data)
        .map_err(|e| Error::other(format!("send error: {}", e)))
    }

    /// 接收数据
    pub fn recv(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(Error::new(
                ErrorKind::NotConnected,
                "Server not connected",
            ));
        }
        self.transport_handler.recv()
        .map_err(|e| Error::other(format!("recv error: {}", e)))
    }

    /// 断开连接
    pub fn disconnect(&mut self) -> Result<()> {
        info!("VirgeServer disconnecting");
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

        if self.connected {
            self.transport_handler.disconnect()?;
            self.connected = false;
        }
        Ok(())
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connected && self.transport_handler.is_connected()
    }
}

impl VirgeServer {
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

impl Read for VirgeServer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.connected {
            return Err(Error::new(
                ErrorKind::NotConnected,
                "Server not connected",
            ));
        }

        match self.read_state {
            ReadState::Idle => {
                self.read_new_message(buf)
            }
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

impl Write for VirgeServer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if !self.connected {
            return Err(Error::new(
                ErrorKind::NotConnected,
                "Server not connected",
            ));
        }

        match self.transport_handler.send(buf) {
            Ok(len) => Ok(len),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Write error: {}", e),
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}


