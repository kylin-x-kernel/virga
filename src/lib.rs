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

pub mod transport;
pub mod client;
pub mod server;

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

#[derive(PartialEq)]
enum ReadState {
    Idle,
    Reading {
        total: usize,
        read: usize,
    },
}
