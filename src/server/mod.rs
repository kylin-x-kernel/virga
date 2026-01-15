//! 服务器模块
//!
//! 提供服务器角色的高级 API。
//!
//! # 职责
//! - 封装服务器的监听逻辑
//! - 处理客户端连接
//! - 提供简洁的发送/接收接口
//! - 管理传输协议选择
//!
//! # 设计思路
//! ```text
//! ┌────────────────────────────────────┐
//! │ VirgeServer                        │
//! │ - transport: Box<dyn Transport>    │
//! │ - config: ServerConfig             │
//! │ - listener: VsockListener          │
//! └────────────────────────────────────┘
//!          │
//!          ├─ listen()
//!          ├─ accept()
//!          ├─ send()
//!          └─ recv()
//! ```
//!
//! # 使用示例
//! ```ignore
//! let mut server = VirgeServer::with_yamux()
//!     .config(ServerConfig { cid: 2, port: 1234, ..Default::default() })
//!     .listen()
//!     .await?;
//!
//! while let Ok(conn) = server.accept().await {
//!     conn.send(vec![1, 2, 3]).await?;
//!     let data = conn.recv().await?;
//! }
//! ```

use crate::error::Result;
use crate::transport::Transport;

/// 服务器配置
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// 服务器监听的 CID
    pub listen_cid: u32,
    
    /// 服务器监听的端口
    pub listen_port: u32,
    
    /// 最大并发连接数
    pub max_connections: usize,
    
    // 其他配置...
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_cid: crate::DEFAULT_SERVER_CID as u32,
            listen_port: crate::DEFAULT_SERVER_PORT as u32,
            max_connections: 100,
        }
    }
}

/// Virga 服务器
///
/// 提供基于选定传输协议的高级服务器接口。
pub struct VirgeServer {
    /// 传输协议实现
    transport: Box<dyn Transport>,
    
    /// 服务器配置
    config: ServerConfig,
    
    /// 监听状态
    listening: bool,
}

impl VirgeServer {
    /// 使用 Yamux 创建服务器
    pub fn with_yamux(config: ServerConfig) -> Self {
        // TODO: 创建 YamuxTransport 实例
        Self {
            transport: Box::new(crate::transport::YamuxTransport::new()),
            config,
            listening: false,
        }
    }
    
    /// 使用 XTransport 创建服务器
    pub fn with_xtransport(config: ServerConfig) -> Self {
        // TODO: 创建 XTransportHandler 实例
        Self {
            transport: Box::new(crate::transport::XTransportHandler::new()),
            config,
            listening: false,
        }
    }
    
    /// 启动监听
    pub async fn listen(&mut self) -> Result<()> {
        log::info!(
            "VirgeServer listening on cid={}, port={}",
            self.config.listen_cid,
            self.config.listen_port
        );
        
        // TODO:
        // 1. 创建 vsock 监听器
        // 2. 初始化传输协议
        // 3. 设置 listening = true
        
        self.transport.connect().await?;
        self.listening = true;
        Ok(())
    }
    
    /// 停止监听
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("VirgeServer stopping");
        
        // TODO: 关闭监听器和所有连接
        self.transport.disconnect().await?;
        self.listening = false;
        Ok(())
    }
    
    /// 发送数据给客户端
    pub async fn send(&mut self, data: Vec<u8>) -> Result<()> {
        if !self.listening {
            return Err(crate::error::VirgeError::Other(
                "Server not listening".to_string(),
            ));
        }
        
        self.transport.send(data).await?;
        Ok(())
    }
    
    /// 从客户端接收数据
    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        if !self.listening {
            return Err(crate::error::VirgeError::Other(
                "Server not listening".to_string(),
            ));
        }
        
        self.transport.recv().await
    }
    
    /// 检查监听状态
    pub fn is_listening(&self) -> bool {
        self.listening && self.transport.is_active()
    }
}
