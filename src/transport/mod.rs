//! 传输协议层模块
//!
//! 定义和实现各种传输协议（yamux、xtransport 等）。
//!
//! # 职责
//! - 定义统一的 Transport trait
//! - 实现具体的传输协议
//! - 屏蔽底层连接差异
//! - 处理协议相关的编码、解码、多路复用等
//!
//! # 设计思路
//! ```text
//! ┌──────────────────────────────────┐
//! │ Transport Trait                  │
//! │ - connect()                      │
//! │ - disconnect()                   │
//! │ - send()                         │
//! │ - recv()                         │
//! └──────────────┬───────────────────┘
//!                │
//!    ┌───────────┼───────────┐
//!    │           │           │
//!    ▼           ▼           ▼
//! Yamux     XTransport    其他
//! ```

pub mod yamux_impl;
pub mod xtransport_impl;

use crate::error::Result;
use std::pin::Pin;
use std::future::Future;

/// 传输协议抽象 trait
///
/// 定义所有传输协议必须实现的接口，隐藏底层协议细节。
///
/// 使用 `Box<dyn Future>` 返回类型以支持异步操作。
pub trait Transport: Send + Sync {
    /// 建立传输连接
    ///
    /// 基于底层 vsock 连接初始化传输协议
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    
    /// 断开传输连接
    ///
    /// 清理协议相关资源
    fn disconnect(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    
    /// 通过该传输发送字节流
    ///
    /// # Arguments
    /// - `data`: 要发送的字节数据
    ///
    /// # Returns
    /// 成功发送返回 Ok，否则返回错误
    fn send(&mut self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    
    /// 通过该传输接收字节流
    ///
    /// # Returns
    /// 返回接收到的字节数据，或错误
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>>;
    
    /// 检查传输是否活跃
    fn is_active(&self) -> bool;
}

// 具体实现模块
pub use yamux_impl::YamuxTransport;
pub use xtransport_impl::XTransportHandler;
