//! 传输协议层模块
//!
//! 定义和实现各种传输协议（yamux、xtransport 等）。
//!
//! # 职责
//! - 定义统一的 Transport trait
//! - 实现具体的传输协议
//! - 直接管理 vsock 连接和协议逻辑
//! - 提供开箱即用的 connect/disconnect/send/recv 接口
//!
//! # 设计思路
//! ```text
//! ┌─────────────────────────────────────┐
//! │ Transport Trait                     │
//! │ - connect(cid, port)               │
//! │ - disconnect()                     │
//! │ - send(data)                       │
//! │ - recv() -> Vec<u8>                │
//! │ - is_connected()                   │
//! └─────────────────┬───────────────────┘
//!                   │
//!       ┌───────────┼───────────┐
//!       │           │           │
//!       ▼           ▼           ▼
//!   YamuxImpl   XTransportImpl   其他
//!   (tokio-vsock)  (vsock)
//! ```

#[cfg(feature = "use-yamux")]
pub mod yamux_impl;
#[cfg(feature = "use-xtransport")]
pub mod xtransport_impl;

use crate::error::Result;
use std::pin::Pin;
use std::future::Future;

/// 传输协议抽象 trait
///
/// 直接封装 vsock 连接和传输协议，提供开箱即用的接口。
/// 每个实现负责管理自己的 vsock 连接和协议状态。
pub trait Transport: Send + Sync {
    /// 建立 vsock 连接并初始化传输协议（客户端模式）
    ///
    /// # Arguments
    /// - `cid`: vsock 连接标识符
    /// - `port`: vsock 端口号
    ///
    /// # Returns
    /// 连接成功返回 Ok，否则返回错误
    fn connect(&mut self, cid: u32, port: u32) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// 从现有 vsock 流初始化传输协议（服务器模式）
    ///
    /// # Arguments
    /// - `stream`: 已建立的 vsock 连接流
    ///
    /// # Returns
    /// 初始化成功返回 Ok，否则返回错误
    fn from_stream(&mut self, _stream: vsock::VsockStream) -> Result<()> {
        // 默认实现：不支持服务器模式
        Err(crate::error::VirgeError::Other("Server mode not supported".to_string()))
    }

    /// 断开连接并清理资源
    fn disconnect(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// 发送数据
    ///
    /// # Arguments
    /// - `data`: 要发送的字节数据
    ///
    /// # Returns
    /// 成功发送返回 Ok，否则返回错误
    fn send(&mut self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// 接收数据
    ///
    /// # Returns
    /// 返回接收到的字节数据，或错误
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>>;

    /// 检查连接是否活跃
    fn is_connected(&self) -> bool;
}

// 具体实现模块
#[cfg(feature = "use-yamux")]
pub use yamux_impl::YamuxTransport;
#[cfg(feature = "use-xtransport")]
pub use xtransport_impl::XTransportHandler;
