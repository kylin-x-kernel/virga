//! 客户端模块
//!
//! 提供客户端角色的高级 API。
//!
//! # 职责
//! - 封装客户端的连接逻辑
//! - 提供简洁的发送/接收接口
//! - 管理传输协议选择
//!
//! # 设计思路
//! ```text
//! ┌────────────────────────────────────┐
//! │ VirgeClient                        │
//! │ - transport: Box<dyn Transport>    │
//! │ - config: ClientConfig             │
//! └────────────────────────────────────┘
//!          │
//!          ├─ connect()
//!          ├─ disconnect()
//!          ├─ send()
//!          └─ recv()
//! ```
//!
//! # 使用示例
//! ```ignore
//! let mut client = VirgeClient::with_yamux()
//!     .config(ClientConfig { cid: 103, port: 1234, ..Default::default() })
//!     .connect()
//!     .await?;
//!
//! client.send(vec![1, 2, 3, 4, 5]).await?;
//! let data = client.recv().await?;
//! 
//! client.disconnect().await?;
//! ```

use crate::error::Result;
use crate::transport::Transport;

/// 客户端配置
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// 服务器 CID
    pub server_cid: u32,
    
    /// 服务器端口
    pub server_port: u32,
    
    /// 连接超时（毫秒）
    pub connect_timeout_ms: u64,
    
    // 其他配置...
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_cid: crate::DEFAULT_SERVER_CID as u32,
            server_port: crate::DEFAULT_SERVER_PORT as u32,
            connect_timeout_ms: 5000,
        }
    }
}

/// Virga 客户端
///
/// 提供基于选定传输协议的高级客户端接口。
pub struct VirgeClient {
    /// 传输协议实现
    transport: Box<dyn Transport>,
    
    /// 客户端配置
    config: ClientConfig,
    
    /// 连接状态
    connected: bool,
}

impl VirgeClient {
    /// 使用 Yamux 创建客户端
    pub fn with_yamux(config: ClientConfig) -> Self {
        // TODO: 创建 YamuxTransport 实例
        Self {
            transport: Box::new(crate::transport::YamuxTransport::new()),
            config,
            connected: false,
        }
    }
    
    /// 使用 XTransport 创建客户端
    pub fn with_xtransport(config: ClientConfig) -> Self {
        // TODO: 创建 XTransportHandler 实例
        Self {
            transport: Box::new(crate::transport::XTransportHandler::new()),
            config,
            connected: false,
        }
    }
    
    /// 建立连接
    pub async fn connect(&mut self) -> Result<()> {
        log::info!(
            "VirgeClient connecting to cid={}, port={}",
            self.config.server_cid,
            self.config.server_port
        );
        
        // TODO:
        // 1. 设置连接参数到 transport
        // 2. 调用 transport.connect()
        // 3. 设置 connected = true
        
        self.transport.connect().await?;
        self.connected = true;
        Ok(())
    }
    
    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        log::info!("VirgeClient disconnecting");
        
        // TODO: 调用 transport.disconnect()
        self.transport.disconnect().await?;
        self.connected = false;
        Ok(())
    }
    
    /// 发送数据
    pub async fn send(&mut self, data: Vec<u8>) -> Result<()> {
        if !self.connected {
            return Err(crate::error::VirgeError::Other(
                "Client not connected".to_string(),
            ));
        }
        
        self.transport.send(data).await?;
        Ok(())
    }
    
    /// 接收数据
    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(crate::error::VirgeError::Other(
                "Client not connected".to_string(),
            ));
        }
        
        self.transport.recv().await
    }
    
    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connected && self.transport.is_active()
    }
}
