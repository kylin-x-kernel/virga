//! 连接层模块
//!
//! 负责 vsock 底层连接的封装。
//!
//! # 职责
//! - 封装 tokio-vsock / vsock 原生 API
//! - 管理连接的生命周期（建立、关闭、错误处理）
//! - 提供统一的连接抽象 trait
//! - 处理连接超时、重试等通用逻辑
//!
//! # 设计思路
//! ```text
//! ┌────────────────────────────────┐
//! │ VsockConnection Trait          │
//! │ - connect()                    │
//! │ - disconnect()                 │
//! │ - read_exact()                 │
//! │ - write_all()                  │
//! │ - is_connected()               │
//! └────────────┬───────────────────┘
//!              │
//!              ▼
//! ┌────────────────────────────────┐
//! │ TokioVsockImpl / VsockImpl      │
//! │ (具体实现)                     │
//! └────────────────────────────────┘
//! ```

use crate::error::Result;

/// Vsock 连接的抽象 trait
///
/// 定义所有连接操作的标准接口，支持多种底层实现（tokio-vsock、vsock 等）。
pub trait VsockConnection: Send + Sync {
    /// 建立 vsock 连接
    ///
    /// # Arguments
    /// - `cid`: 连接标识符
    /// - `port`: 端口号
    ///
    /// # Returns
    /// 连接成功返回 Ok，否则返回错误
    fn connect(&mut self, cid: u32, port: u32) -> impl std::future::Future<Output = Result<()>>;
    
    /// 断开连接
    fn disconnect(&mut self) -> impl std::future::Future<Output = Result<()>>;
    
    /// 从连接中读取指定字节数
    ///
    /// 如果无法读取到指定字节数，返回错误
    fn read_exact(&mut self, buf: &mut [u8]) -> impl std::future::Future<Output = Result<()>>;
    
    /// 向连接中写入所有数据
    ///
    /// 确保所有数据都被写入，否则返回错误
    fn write_all(&mut self, buf: &[u8]) -> impl std::future::Future<Output = Result<()>>;
    
    /// 检查连接是否还活跃
    fn is_connected(&self) -> bool;
}

// TODO: 实现 TokioVsockImpl
// pub struct TokioVsockImpl { ... }
// impl VsockConnection for TokioVsockImpl { ... }

// TODO: 如果需要支持原生 vsock，实现 VsockImpl
// pub struct VsockImpl { ... }
// impl VsockConnection for VsockImpl { ... }
