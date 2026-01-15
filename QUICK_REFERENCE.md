# Virga 快速参考

## 项目结构一览

```
virga/
├── src/
│   ├── lib.rs                      # 主入口，导出公共 API
│   ├── error/mod.rs                # 错误定义 → VirgeError
│   ├── connection/mod.rs           # 连接 trait → VsockConnection
│   ├── transport/
│   │   ├── mod.rs                  # 协议 trait → Transport
│   │   ├── yamux_impl/mod.rs       # Yamux 实现 ✏️ 待完成
│   │   └── xtransport_impl/mod.rs  # XTransport 实现 ✏️ 待完成
│   ├── client/mod.rs               # 客户端 API → VirgeClient
│   └── server/mod.rs               # 服务器 API → VirgeServer
│
├── Cargo.toml                       # 项目配置，特征和依赖
├── Cargo.lock                       # 依赖锁文件
│
├── ARCHITECTURE.md                 # 📖 详细设计文档（强烈推荐）
├── DESIGN_SUMMARY.md               # 📋 设计完成总结
├── EXAMPLES.md                     # 💡 使用示例和最佳实践
├── TODO.md                         # 📝 开发清单和路线
└── README.md                       # （待编写）
```

## 核心概念速记

### 分层架构
```
应用层 (Application)
    ↓ 使用
协议层 (Protocol) ← 可扩展
    ↓ 使用
连接层 (Connection) ← 可扩展
    ↓ 使用
错误层 (Error)
```

### 关键 Trait

| Trait | 位置 | 职责 | 实现数 |
|-------|------|------|--------|
| `Transport` | `transport/mod.rs` | 传输协议抽象 | 2 (Yamux, XTransport) |
| `VsockConnection` | `connection/mod.rs` | vsock 连接抽象 | 0 (待实现) |

### 关键类型

| 类型 | 位置 | 说明 |
|------|------|------|
| `VirgeClient` | `client/mod.rs` | 客户端，工厂：`with_yamux()`、`with_xtransport()` |
| `VirgeServer` | `server/mod.rs` | 服务器，工厂：`with_yamux()`、`with_xtransport()` |
| `VirgeError` | `error/mod.rs` | 统一错误类型 |
| `ClientConfig` | `client/mod.rs` | 客户端配置 |
| `ServerConfig` | `server/mod.rs` | 服务器配置 |

## 编译命令速查

```bash
# 基础
cargo build --no-default-features

# 启用 Yamux
cargo build --no-default-features --features "use-yamux"

# 启用 XTransport
cargo build --no-default-features --features "use-xtransport"

# 启用两者
cargo build --no-default-features --features "use-yamux use-xtransport"

# 测试（所有特征）
cargo test --no-default-features --features "use-yamux use-xtransport"

# 生成文档
cargo doc --no-deps --open

# 代码检查
cargo clippy --no-default-features --features "use-yamux use-xtransport"

# 格式检查
cargo fmt --check
```

## API 速查

### 客户端用法

```rust
// 创建
let config = ClientConfig::default();
let mut client = VirgeClient::with_yamux(config);
// 或
let mut client = VirgeClient::with_xtransport(config);

// 连接
client.connect().await?;

// 发送
client.send(vec![1, 2, 3]).await?;

// 接收
let data = client.recv().await?;

// 检查状态
if client.is_connected() { ... }

// 断开
client.disconnect().await?;
```

### 服务器用法

```rust
// 创建
let config = ServerConfig::default();
let mut server = VirgeServer::with_yamux(config);

// 监听
server.listen().await?;

// 接收
let data = server.recv().await?;

// 发送
server.send(response).await?;

// 检查状态
if server.is_listening() { ... }

// 停止
server.stop().await?;
```

## 错误处理

```rust
use virga::error::VirgeError;

match operation.await {
    Ok(result) => println!("成功: {:?}", result),
    Err(VirgeError::ConnectionError(msg)) => eprintln!("连接错误: {}", msg),
    Err(VirgeError::TransportError(msg)) => eprintln!("传输错误: {}", msg),
    Err(VirgeError::ConfigError(msg)) => eprintln!("配置错误: {}", msg),
    Err(e) => eprintln!("未知错误: {}", e),
}
```

## 特征配置

### Cargo.toml 中的 feature 定义

```toml
[features]
default = []
use-yamux = ["yamux", "tokio", "tokio-util", "tokio-vsock", "futures"]
use-xtransport = ["vsock", "xtransport"]
```

### 条件编译

```rust
#[cfg(feature = "use-yamux")]
fn foo() { ... }

#[cfg(all(feature = "use-yamux", feature = "use-xtransport"))]
fn bar() { ... }
```

## 日志使用

```rust
// 初始化
env_logger::init();

// 记录
log::info!("信息级别");
log::debug!("调试级别");
log::warn!("警告级别");
log::error!("错误级别");

// 运行时控制
RUST_LOG=debug cargo run
RUST_LOG=virga=info cargo run
```

## 模块树状图

```
virga
├── error
│   └── VirgeError
│       ├── ConnectionError
│       ├── TransportError
│       ├── ConfigError
│       ├── IoError
│       └── Other
├── connection
│   └── VsockConnection (Trait)
│       ├── connect()
│       ├── disconnect()
│       ├── read_exact()
│       ├── write_all()
│       └── is_connected()
├── transport
│   ├── Transport (Trait)
│   │   ├── connect()
│   │   ├── disconnect()
│   │   ├── send()
│   │   ├── recv()
│   │   └── is_active()
│   ├── yamux_impl
│   │   └── YamuxTransport
│   └── xtransport_impl
│       └── XTransportHandler
├── client
│   ├── ClientConfig
│   └── VirgeClient
│       ├── with_yamux()
│       ├── with_xtransport()
│       ├── connect()
│       ├── disconnect()
│       ├── send()
│       ├── recv()
│       └── is_connected()
└── server
    ├── ServerConfig
    └── VirgeServer
        ├── with_yamux()
        ├── with_xtransport()
        ├── listen()
        ├── stop()
        ├── send()
        ├── recv()
        └── is_listening()
```

## 文档快速导航

| 需求 | 文档 |
|------|------|
| 理解整体架构 | `ARCHITECTURE.md` |
| 查看使用示例 | `EXAMPLES.md` |
| 了解设计决策 | `DESIGN_SUMMARY.md` |
| 查找待实现项 | `TODO.md` |
| 生成 API 文档 | `cargo doc --no-deps --open` |

## 常见问题速答

**Q: 如何同时使用 Yamux 和 XTransport？**
A: 在 feature 中同时启用两者：`--features "use-yamux use-xtransport"`

**Q: 如何添加新的传输协议？**
A: 实现 `Transport` trait，参考 `EXAMPLES.md` 中的扩展指南。

**Q: 如何添加新的连接类型？**
A: 实现 `VsockConnection` trait，参考 TODO.md 中的 Phase 2。

**Q: 当前哪些已完成，哪些待完成？**
A: 框架设计完成（Phase 1），具体实现待完成（Phase 2-6），见 `TODO.md`。

**Q: 如何运行示例？**
A: 实现完成后，运行 `cargo run --example client_yamux`。

## 实现进度

```
Phase 1: 基础框架      ✅ 100%
Phase 2: 底层实现      ⏳  0%
Phase 3: 协议实现      ⏳  0%
Phase 4: 应用层完善    ⏳  0%
Phase 5: 测试          ⏳  0%
Phase 6: 文档优化      ⏳  0%
```

## 建议的学习路径

1. 阅读本文档（5 分钟）
2. 阅读 `ARCHITECTURE.md`（15 分钟）
3. 查看源代码注释（20 分钟）
4. 阅读 `EXAMPLES.md`（10 分钟）
5. 按 `TODO.md` 开始实现（数小时）

## 有用的快捷键

```bash
# 快速构建并检查（Yamux）
cargo build --no-default-features --features "use-yamux" && cargo clippy --no-default-features --features "use-yamux"

# 快速构建并检查（两者）
cargo build --no-default-features --features "use-yamux use-xtransport" && cargo clippy --no-default-features --features "use-yamux use-xtransport"

# 完整检查
cargo fmt && cargo build --no-default-features --features "use-yamux use-xtransport" && cargo clippy --no-default-features --features "use-yamux use-xtransport" && cargo test --no-default-features --features "use-yamux use-xtransport"
```

---

**最后更新**: 2026-01-15
**框架设计状态**: ✅ 完成（待实现）
