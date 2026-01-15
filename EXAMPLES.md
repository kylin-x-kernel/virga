# Virga 框架使用示例

本文档通过具体示例说明如何使用 Virga 框架。

## 1. 客户端示例

### 1.1 使用 Yamux 传输的客户端

```rust
use virga::client::{VirgeClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 创建客户端配置
    let config = ClientConfig {
        server_cid: 103,
        server_port: 1234,
        connect_timeout_ms: 5000,
    };
    
    // 使用 Yamux 传输协议创建客户端
    let mut client = VirgeClient::with_yamux(config);
    
    // 建立连接
    client.connect().await?;
    println!("客户端连接成功");
    
    // 发送数据
    let data_to_send = vec![1, 2, 3, 4, 5, 6, 7, 8];
    client.send(data_to_send).await?;
    println!("数据发送成功");
    
    // 接收数据
    let received_data = client.recv().await?;
    println!("接收数据: {:?}", received_data);
    
    // 断开连接
    client.disconnect().await?;
    println!("客户端断开连接");
    
    Ok(())
}
```

### 1.2 使用 XTransport 传输的客户端

```rust
use virga::client::{VirgeClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let config = ClientConfig {
        server_cid: 103,
        server_port: 1234,
        connect_timeout_ms: 5000,
    };
    
    // 使用 XTransport 传输协议创建客户端
    let mut client = VirgeClient::with_xtransport(config);
    
    client.connect().await?;
    client.send(vec![1, 2, 3]).await?;
    let data = client.recv().await?;
    client.disconnect().await?;
    
    Ok(())
}
```

## 2. 服务器示例

### 2.1 使用 Yamux 传输的服务器

```rust
use virga::server::{VirgeServer, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 创建服务器配置
    let config = ServerConfig {
        listen_cid: 2,  // Host 监听 CID 为 2
        listen_port: 1234,
        max_connections: 100,
    };
    
    // 使用 Yamux 传输协议创建服务器
    let mut server = VirgeServer::with_yamux(config);
    
    // 启动监听
    server.listen().await?;
    println!("服务器启动监听");
    
    // 处理客户端请求
    loop {
        // 接收客户端数据
        match server.recv().await {
            Ok(data) => {
                println!("接收数据: {:?}", data);
                
                // 回复客户端
                let response = vec![42, 42, 42];
                server.send(response).await?;
            }
            Err(e) => {
                eprintln!("接收数据出错: {}", e);
                break;
            }
        }
    }
    
    server.stop().await?;
    Ok(())
}
```

### 2.2 使用 XTransport 传输的服务器

```rust
use virga::server::{VirgeServer, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let config = ServerConfig::default();
    let mut server = VirgeServer::with_xtransport(config);
    
    server.listen().await?;
    
    loop {
        let data = server.recv().await?;
        server.send(data).await?;
    }
}
```

## 3. 特征使用示例

### 3.1 仅启用 Yamux

```bash
# 只编译 yamux 实现
cargo build --no-default-features --features "use-yamux"
```

### 3.2 仅启用 XTransport

```bash
# 只编译 xtransport 实现
cargo build --no-default-features --features "use-xtransport"
```

### 3.3 同时启用两种传输

```bash
# 同时编译两种实现
cargo build --no-default-features --features "use-yamux use-xtransport"
```

## 4. 条件编译示例

如果需要在代码中根据不同特征条件编译，可以使用 `#[cfg(feature = "...")]`：

```rust
#[cfg(feature = "use-yamux")]
fn create_yamux_client() {
    let client = VirgeClient::with_yamux(ClientConfig::default());
    // ...
}

#[cfg(feature = "use-xtransport")]
fn create_xtransport_client() {
    let client = VirgeClient::with_xtransport(ClientConfig::default());
    // ...
}

#[cfg(all(feature = "use-yamux", feature = "use-xtransport"))]
fn create_multi_protocol_client() {
    // 同时支持两种协议
}
```

## 5. 错误处理示例

```rust
use virga::error::VirgeError;

match client.connect().await {
    Ok(_) => println!("连接成功"),
    Err(VirgeError::ConnectionError(msg)) => {
        eprintln!("连接错误: {}", msg);
    }
    Err(VirgeError::TransportError(msg)) => {
        eprintln!("传输错误: {}", msg);
    }
    Err(e) => {
        eprintln!("未知错误: {}", e);
    }
}
```

## 6. 日志示例

所有重要操作都通过 `log` crate 记录，可以通过环境变量控制日志级别：

```bash
# 显示所有日志
RUST_LOG=debug cargo run

# 仅显示 virga 的日志
RUST_LOG=virga=debug cargo run

# 显示 info 及以上级别的日志
RUST_LOG=info cargo run
```

## 7. 实现自定义传输协议

如果需要添加新的传输协议，步骤如下：

1. 在 `src/transport/` 下创建新目录，例如 `quic_impl/`
2. 实现 `Transport` trait：

```rust
// src/transport/quic_impl/mod.rs
use crate::transport::Transport;
use crate::error::Result;
use std::pin::Pin;
use std::future::Future;

pub struct QuicTransport {
    active: bool,
}

impl Transport for QuicTransport {
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            log::info!("QUIC transport connecting...");
            self.active = true;
            Ok(())
        })
    }
    
    // 实现其他方法...
    
    fn is_active(&self) -> bool {
        self.active
    }
}
```

3. 在 `src/transport/mod.rs` 中导出
4. 在 `Cargo.toml` 中添加新 feature
5. 在 `VirgeClient` 和 `VirgeServer` 中添加 `with_quic()` 方法

## 8. 最佳实践

- 总是使用 `?` 操作符处理错误，避免 `unwrap()`
- 启用日志记录以便调试：`env_logger::init()`
- 为不同的传输场景选择合适的协议：
  - Yamux：多并发、多虚拟流场景
  - XTransport：低延迟、单流场景
- 根据需要添加超时和重试机制
- 定期检查连接状态：`client.is_connected()`、`server.is_listening()`
