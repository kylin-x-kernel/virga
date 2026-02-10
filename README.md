# Virga

一个基于 VSock 的通信库，支持 Yamux 和 XTransport 协议。

## 特性

- 🚀 基于 VSock 的高性能通信
- 🔄 支持多种传输协议（Yamux、XTransport）
- 🏗️ 客户端/服务器架构
- 📦 默认使用 Yamux 协议
- 🔧 灵活的配置选项
- 💡 同步 API，内部异步驱动

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
virga = { git = "https://github.com/your-repo/virga.git" }
# 或指定协议
virga = { git = "https://github.com/your-repo/virga.git", features = ["use-yamux"] }
```

## 快速开始

### 方式一：使用 `send()`/`recv()` API

#### 客户端

```rust
use virga::client::{VirgeClient, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置：server_cid, server_port, chunk_size, is_ack
    let config = ClientConfig::new(103, 1234, 1024, false);
    let mut client = VirgeClient::new(config);
    
    // 建立连接
    client.connect()?;

    // 发送数据
    let data = vec![1; 512];
    let sent = client.send(data)?;
    println!("Sent {} bytes", sent);

    // 接收数据
    let received = client.recv()?;
    println!("Received {} bytes", received.len());

    // 断开连接
    client.disconnect()?;
    Ok(())
}
```

#### 服务器

```rust
use virga::server::{ServerManager, ServerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置：listen_cid, listen_port, chunk_size, is_ack
    let config = ServerConfig::new(0xFFFFFFFF, 1234, 1024, false);
    
    let mut manager = ServerManager::new(config);
    manager.start()?;

    // 接受连接
    if let Ok(mut server) = manager.accept() {
        println!("New client connected");

        // 接收数据
        let data = server.recv()?;
        println!("Received {} bytes", data.len());
        
        // 回显数据
        server.send(data)?;
    }

    Ok(())
}
```

### 方式二：使用 `Read`/`Write` trait

客户端和服务器同样实现了标准的 `std::io::Read` 和 `std::io::Write` trait，可用于流式读写：

```rust
use std::io::{Read, Write};
use virga::client::{VirgeClient, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::new(103, 1234, 1024, false);
    let mut client = VirgeClient::new(config);
    client.connect()?;

    // 使用 Write trait
    let data = vec![1; 512];
    client.write_all(&data)?;

    // 使用 Read trait（分块读取）
    let mut buf = [0u8; 64];
    loop {
        let n = client.read(&mut buf)?;
        if n == 0 || client.no_has_data() {
            break;
        }
        println!("Read {} bytes", n);
    }

    client.disconnect()?;
    Ok(())
}
```

## 配置

### ClientConfig

```rust
use virga::client::ClientConfig;

// 方式1：使用 new 构造
let config = ClientConfig::new(
    103,        // server_cid: 服务器 CID
    1234,       // server_port: 服务器端口
    1024,       // chunk_size: 数据块大小
    false       // is_ack: 是否启用 ACK
);

// 方式2：使用默认配置
let config = ClientConfig::default();
```

### ServerConfig

```rust
use virga::server::ServerConfig;

// 方式1：使用 new 构造
let config = ServerConfig::new(
    0xFFFFFFFF, // listen_cid: 监听 CID（VMADDR_CID_ANY）
    1234,       // listen_port: 监听端口
    1024,       // chunk_size: 数据块大小
    false       // is_ack: 是否启用 ACK
);

// 方式2：使用默认配置
let config = ServerConfig::default();
```

## 协议选择

Virga 支持两种传输协议，通过 Cargo features 选择：

### Yamux（默认）

多路复用传输协议，基于 libp2p yamux 实现。

```toml
[dependencies]
virga = { version = "0.1.0", features = ["use-yamux"] }
# 或者不指定 features（默认启用 yamux）
virga = { version = "0.1.0" }
```

### XTransport

轻量级传输协议，适合简单场景。

```toml
[dependencies]
virga = { version = "0.1.0", features = ["use-xtransport"] }
```

## API 说明

### VirgeClient

| 方法 | 说明 |
|------|------|
| `new(config)` | 创建客户端实例 |
| `connect()` | 建立连接 |
| `send(data)` | 发送数据，返回发送字节数 |
| `recv()` | 接收数据，返回接收的数据 |
| `disconnect()` | 断开连接 |
| `is_connected()` | 检查连接状态 |
| `no_has_data()` | 检查是否还有未读数据 |

### VirgeServer

| 方法 | 说明 |
|------|------|
| `send(data)` | 发送数据，返回发送字节数 |
| `recv()` | 接收数据，返回接收的数据 |
| `disconnect()` | 断开连接 |
| `is_connected()` | 检查连接状态 |
| `no_has_data()` | 检查是否还有未读数据 |

### ServerManager

| 方法 | 说明 |
|------|------|
| `new(config)` | 创建服务器管理器 |
| `start()` | 开始监听 |
| `accept()` | 接受新连接，返回 VirgeServer |
| `stop()` | 停止监听 |
| `is_running()` | 检查是否在运行 |

## 许可证

Apache-2.0
