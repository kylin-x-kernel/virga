# Virga

一个基于 VSock 的通信库，支持 Yamux 和 XTransport 协议。

## 特性

- 🚀 基于 VSock 的高性能通信
- 🔄 支持多种传输协议（XTransport、Yamux）
- 🏗️ 客户端/服务器架构
- 📦 默认使用 XTransport 协议
- 🔧 灵活的配置选项

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
virga = { git = "https://github.com/your-repo/virga.git", features = ["use-xtransport"] }
```

## 快速开始

### 客户端示例

```rust
use virga::client::{VirgeClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = ClientConfig::default();
    let mut client = VirgeClient::new(config);
    client.connect().await?;

    client.send(vec![1, 2, 3, 4, 5]).await?;
    let data = client.recv().await?;
    println!("{:?}", data);

    client.disconnect().await?;
    Ok(())
}
```

### 服务器示例

```rust
use virga::server::{ServerManager, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let mut manager = ServerManager::new(config);
    manager.start().await?;

    while let Ok(mut server) = manager.accept().await {
        println!("there is a new virgeserver");
        tokio::spawn(async move {
            // 处理接收数据
            let data_result = server.recv().await;
            let data = match data_result {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("接收数据失败: {}", e);
                    return;  // 直接返回，不继续执行
                }
            };

            // 处理发送数据
            if let Err(e) = server.send(data).await {
                eprintln!("发送数据失败: {}", e);
            }

            // 处理断开连接
            if let Err(e) = server.disconnect().await {
                eprintln!("断开连接失败: {}", e);
            }
        });
    }

    Ok(())
}
```

## 配置

### 客户端配置

```rust
use virga::client::ClientConfig;

let config = ClientConfig {
    server_cid: 103,  // 服务器 CID，默认为 103
    server_port: 1234,  // 服务器端口，默认为 1234
    chunk_size: 1024,  // 数据块大小，默认为 1024
    is_ack: false,  // 是否启用 ACK，默认为 false
};
```

### 服务器配置

```rust
use virga::server::ServerConfig;

let config = ServerConfig {
    listen_cid: 0xFFFFFFFF,  // 监听 CID，默认为 VMADDR_CID_ANY (0xFFFFFFFF)
    listen_port: 1234,  // 监听端口，默认为 1234
    chunk_size: 1024,  // 数据块大小，默认为 1024
    is_ack: false,  // 是否启用 ACK，默认为 false
};
```

## 协议选择

Virga 支持两种传输协议：

### XTransport（默认）

轻量级传输协议，适合大多数应用场景。

```toml
[dependencies]
virga = { version = "0.1.0", features = ["use-xtransport"] }
```

### Yamux

多路复用传输协议，适合需要并发流的应用。

```toml
[dependencies]
virga = { version = "0.1.0", features = ["use-yamux"] }
```

## 构建

```bash
# 构建项目（默认启用 XTransport）
cargo build

# 仅启用 XTransport（包含必要的 tokio 依赖）
cargo build --no-default-features --features use-xtransport

# 仅启用 Yamux
cargo build --no-default-features --features use-yamux

# 同时启用两种协议
cargo build --no-default-features --features "use-xtransport use-yamux"
```

## 运行示例

```bash
# 运行客户端示例（使用 XTransport）
cargo run --example test_client --features use-xtransport --no-default-features

# 运行服务器示例（使用 XTransport）
cargo run --example test_server --features use-xtransport --no-default-features

# 或者同时启用两种协议运行
cargo run --example test_client --features "use-xtransport use-yamux"
cargo run --example test_server --features "use-xtransport use-yamux"
```

## 文档

生成完整的 API 文档：

```bash
cargo doc --no-deps --open
```

## 许可证

本项目采用 MIT 许可证。
