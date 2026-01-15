# Virga 架构设计文档

## 1. 概述

Virga 是一个基于 vsock 的字节流传输库，支持多种传输协议（yamux、xtransport）。采用分层架构设计，实现关注点分离（Separation of Concerns）和可扩展性。

## 2. 分层架构

```
┌──────────────────────────────────────────────────────────────┐
│  应用层（Application Layer）                                  │
│  VirgeClient / VirgeServer                                   │
│  职责：高级 API，用户直接使用                                 │
│  依赖：协议层                                                 │
└──────────────────────┬───────────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────────┐
│  协议层（Protocol Layer）                                     │
│  Transport Trait + 具体实现（Yamux、XTransport）             │
│  职责：屏蔽传输协议差异，提供统一接口                        │
│  依赖：连接层                                                 │
└──────────────────────┬───────────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────────┐
│  连接层（Connection Layer）                                   │
│  VsockConnection Trait + 底层实现                            │
│  职责：vsock 连接管理，生命周期控制                          │
│  依赖：错误层、底层库（tokio-vsock、vsock）                 │
└──────────────────────┬───────────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────────┐
│  错误层（Error Layer）                                        │
│  VirgeError 统一错误类型                                      │
│  职责：定义和处理所有错误                                     │
└──────────────────────────────────────────────────────────────┘
```

## 3. 模块说明

### 3.1 错误层（`src/error/mod.rs`）

定义统一的错误类型 `VirgeError`，包括：
- `ConnectionError`：连接相关错误
- `TransportError`：传输协议错误
- `ConfigError`：配置错误
- `IoError`：IO 错误
- `Other`：其他错误

所有错误都实现 `std::error::Error` trait，便于 `?` 操作符使用。

**何时扩展**：
- 添加新的错误类别时
- 需要更详细的错误上下文时

### 3.2 连接层（`src/connection/mod.rs`）

定义 `VsockConnection` trait，标准化 vsock 操作：

```rust
pub trait VsockConnection: Send + Sync {
    fn connect(&mut self, cid: u32, port: u32) -> impl Future<Output = Result<()>>;
    fn disconnect(&mut self) -> impl Future<Output = Result<()>>;
    fn read_exact(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<()>>;
    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = Result<()>>;
    fn is_connected(&self) -> bool;
}
```

**实现候选**：
- `TokioVsockImpl`：基于 tokio-vsock 的异步实现
- `VsockImpl`：基于原生 vsock 的实现

**何时扩展**：
- 支持不同的底层库时（如 mio、async-std）
- 需要特殊的连接管理（如连接池、重试机制）时

### 3.3 协议层（`src/transport/mod.rs` 及子目录）

定义 `Transport` trait，抽象传输协议：

```rust
pub trait Transport: Send + Sync {
    fn connect(&mut self) -> impl Future<Output = Result<()>>;
    fn disconnect(&mut self) -> impl Future<Output = Result<()>>;
    fn send(&mut self, data: Vec<u8>) -> impl Future<Output = Result<()>>;
    fn recv(&mut self) -> impl Future<Output = Result<Vec<u8>>>;
    fn is_active(&self) -> bool;
}
```

**具体实现**：
- `YamuxTransport`（`src/transport/yamux_impl/mod.rs`）
  - 特点：多路复用，支持并发流
  - 使用场景：需要多个独立虚拟流的应用
  
- `XTransportHandler`（`src/transport/xtransport_impl/mod.rs`）
  - 特点：轻量级，针对 vsock 优化
  - 使用场景：对延迟敏感的应用

**何时扩展**：
- 支持新的传输协议时（如 QUIC、mtp）
- 需要特殊的传输语义（如单向/双向、顺序保证等）

### 3.4 应用层（`src/client/mod.rs` 和 `src/server/mod.rs`）

提供高级 API，供用户直接使用。

#### 3.4.1 客户端（`VirgeClient`）

职责：
- 初始化传输协议
- 建立到服务器的连接
- 提供 `send()`/`recv()` 接口

使用示例：
```rust
let mut client = VirgeClient::with_yamux(ClientConfig::default());
client.connect().await?;
client.send(vec![1, 2, 3]).await?;
let data = client.recv().await?;
client.disconnect().await?;
```

#### 3.4.2 服务器（`VirgeServer`）

职责：
- 监听 vsock 连接
- 处理客户端连接
- 提供 `send()`/`recv()` 接口

使用示例：
```rust
let mut server = VirgeServer::with_yamux(ServerConfig::default());
server.listen().await?;
loop {
    let data = server.recv().await?;
    server.send(data).await?;
}
```

**何时扩展**：
- 需要更高级的功能（如连接管理、负载均衡）
- 支持新的使用模式

## 4. 特征（Features）与依赖管理

项目定义了两个 optional 特征：

```toml
[features]
default = []
use-yamux = ["yamux", "tokio", "tokio-util", "tokio-vsock", "futures"]
use-xtranport = ["vsock", "xtranport"]

[dependencies]
yamux = { optional = true, ... }
xtranport = { optional = true, ... }
tokio = { optional = true, ... }
# 等等
```

**好处**：
- 按需拉取依赖，减少编译时间和二进制大小
- 不同特征可以独立启用或混合启用

**使用示例**：
```bash
# 仅支持 yamux
cargo build --no-default-features --features "use-yamux"

# 仅支持 xtransport
cargo build --no-default-features --features "use-xtranport"

# 两者都支持
cargo build --no-default-features --features "use-yamux use-xtranport"
```

## 5. 数据流示意

### 5.1 客户端发送数据流

```
应用层
  │
  ├─ client.send(data)
  │
应用层/协议层边界
  │
  ├─ Transport::send(data)
  │   ├─ 数据编码（如需要）
  │   └─ 虚拟流分配（yamux 情况下）
  │
协议层/连接层边界
  │
  ├─ VsockConnection::write_all(encoded_data)
  │   ├─ vsock 缓冲
  │   └─ 网络传输
  │
底层 vsock 库
  └─ 最终传输到 Host Kernel
```

### 5.2 客户端接收数据流

```
底层 vsock 库
  │
  └─ 从 Host Kernel 接收
  │
连接层
  │
  ├─ VsockConnection::read_exact(buf)
  │   ├─ 等待数据到达
  │   └─ 读入缓冲
  │
协议层
  │
  ├─ Transport::recv()
  │   ├─ 虚拟流读取（yamux 情况下）
  │   ├─ 数据解码（如需要）
  │   └─ 返回原始数据
  │
应用层
  └─ data = client.recv().await?
```

## 6. 配置与定制

### 6.1 ClientConfig

```rust
pub struct ClientConfig {
    pub server_cid: u32,              // 远程服务器 CID
    pub server_port: u32,             // 远程服务器端口
    pub connect_timeout_ms: u64,      // 连接超时
    // ... 可扩展字段
}
```

### 6.2 ServerConfig

```rust
pub struct ServerConfig {
    pub listen_cid: u32,              // 本地监听 CID
    pub listen_port: u32,             // 本地监听端口
    pub max_connections: usize,       // 最大并发连接数
    // ... 可扩展字段
}
```

## 7. 扩展点

### 7.1 添加新的传输协议

1. 在 `src/transport/` 下创建新目录（如 `quic_impl`）
2. 实现 `Transport` trait
3. 在 `src/transport/mod.rs` 中导出
4. 在 `Cargo.toml` 中添加新的 feature
5. 在 `VirgeClient` 和 `VirgeServer` 中添加 `with_xxx()` 方法

### 7.2 添加连接池

在连接层上添加连接管理逻辑：
- 维护空闲连接池
- 自动重连机制
- 健康检查

### 7.3 添加中间件

在应用层之下、协议层之上插入中间件：
- 数据压缩
- 加密
- 速率限制

## 8. 实现路线

**Phase 1：基础框架**（当前）
- ✅ 错误定义
- ✅ 连接层 trait 定义
- ✅ 协议层 trait 定义
- ✅ 应用层 trait 定义
- ⏳ 底层实现（TokioVsockImpl、YamuxTransport 等）

**Phase 2：核心实现**
- [ ] 实现 TokioVsockImpl
- [ ] 实现 YamuxTransport
- [ ] 实现 XTransportHandler
- [ ] 编写单元测试
- [ ] 编写集成测试

**Phase 3：优化与扩展**
- [ ] 连接池
- [ ] 自动重连
- [ ] 监控与日志
- [ ] 性能优化
- [ ] 文档完善

## 9. 编码规范

- 所有公开的 trait 和结构都需要详细的文档注释
- 使用 `log` crate 记录重要事件（info、debug）
- 所有异步函数都应该在文档中标注 `#[tokio::main]` 或其他运行时要求
- 错误处理优先使用 `?` 操作符而不是 `unwrap()`

## 10. 参考

- vsock：https://github.com/stefano-garzarella/vsock
- yamux：https://github.com/libp2p/rust-yamux
- xtransfer：https://github.com/kylin-x-kernel/xtransfer
