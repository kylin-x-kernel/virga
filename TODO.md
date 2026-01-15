# Virga 开发清单

本文档列出所有待实现的功能，按优先级和逻辑顺序排列。

## Phase 1: 基础框架（✅ 完成）

- [x] 创建错误层模块 (`src/error/mod.rs`)
- [x] 定义 `VirgeError` 和 `Result` 类型
- [x] 创建连接层模块 (`src/connection/mod.rs`)
- [x] 定义 `VsockConnection` trait
- [x] 创建协议层模块 (`src/transport/mod.rs`)
- [x] 定义 `Transport` trait
- [x] 创建 Yamux 实现模块 (`src/transport/yamux_impl/mod.rs`)
- [x] 创建 XTransport 实现模块 (`src/transport/xtransport_impl/mod.rs`)
- [x] 创建客户端模块 (`src/client/mod.rs`)
- [x] 创建服务器模块 (`src/server/mod.rs`)
- [x] 配置特征和依赖管理
- [x] 编写架构文档 (`ARCHITECTURE.md`)
- [x] 编写使用示例 (`EXAMPLES.md`)

## Phase 2: 底层连接实现

### 2.1 Tokio Vsock 实现

**优先级**: 高

- [ ] 创建 `src/connection/tokio_vsock_impl.rs`
- [ ] 实现 `TokioVsockImpl` 结构体
- [ ] 实现 `VsockConnection` trait for `TokioVsockImpl`
  - [ ] `connect()` - 使用 tokio-vsock 建立连接
  - [ ] `disconnect()` - 关闭连接
  - [ ] `read_exact()` - 完全读取
  - [ ] `write_all()` - 完全写入
  - [ ] `is_connected()` - 检查连接状态
- [ ] 添加连接超时支持
- [ ] 添加错误处理和日志

**测试**:
```bash
cargo test connection::tokio_vsock_impl
```

### 2.2 原生 Vsock 实现（可选）

**优先级**: 低

- [ ] 创建 `src/connection/native_vsock_impl.rs`
- [ ] 实现 `NativeVsockImpl` 结构体
- [ ] 实现 `VsockConnection` trait
- [ ] 支持异步操作（使用 tokio::task::block_in_place 或类似）

## Phase 3: 传输协议实现

### 3.1 Yamux 传输实现

**优先级**: 高

**文件**: `src/transport/yamux_impl/mod.rs`

**任务列表**:
- [ ] 导入 yamux 库
- [ ] 实现 `YamuxTransport::new()` - 初始化
  - [ ] 创建 vsock 连接
  - [ ] 配置 yamux 参数
- [ ] 实现 `Transport::connect()`
  - [ ] 建立底层 vsock 连接
  - [ ] 初始化 yamux 多路复用
  - [ ] 设置 active 标志
- [ ] 实现 `Transport::disconnect()`
  - [ ] 关闭所有虚拟流
  - [ ] 关闭底层连接
- [ ] 实现 `Transport::send()`
  - [ ] 打开或获取虚拟流
  - [ ] 写入数据
  - [ ] 处理错误
- [ ] 实现 `Transport::recv()`
  - [ ] 从虚拟流读取
  - [ ] 处理多个流的情况
  - [ ] 返回数据
- [ ] 添加流管理（打开/关闭/复用）
- [ ] 添加错误处理和日志

**关键代码片段**（提示）:
```rust
// 连接和初始化
let (client, mut server) = yamux::Control::connect(mode, vsock_conn, config);

// 发送
let mut stream = client.open_stream().await?;
stream.write_all(&data).await?;

// 接收
while let Some(stream) = server.accept().await? {
    let data = stream.read_to_end(&mut buf).await?;
}
```

**测试**:
```bash
cargo test --features "use-yamux" transport::yamux_impl
```

### 3.2 XTransport 实现

**优先级**: 高

**文件**: `src/transport/xtransport_impl/mod.rs`

**任务列表**:
- [ ] 导入 xtransport 库
- [ ] 了解 xtransport API（frame 格式、编码等）
- [ ] 实现 `XTransportHandler::new()`
- [ ] 实现 `Transport::connect()`
  - [ ] 建立 vsock 连接
  - [ ] 初始化 xtransport 处理器
- [ ] 实现 `Transport::disconnect()`
- [ ] 实现 `Transport::send()`
  - [ ] 编码数据（如需要）
  - [ ] 发送到底层连接
- [ ] 实现 `Transport::recv()`
  - [ ] 从底层连接接收
  - [ ] 解码数据
- [ ] 处理帧格式和协议细节
- [ ] 错误处理和日志

**测试**:
```bash
cargo test --features "use-xtransport" transport::xtransport_impl
```

## Phase 4: 应用层实现

### 4.1 客户端完善

**优先级**: 中

**文件**: `src/client/mod.rs`

**任务列表**:
- [ ] 完善 `VirgeClient::with_yamux()`
  - [ ] 正确初始化 `YamuxTransport`
  - [ ] 传递配置参数
- [ ] 完善 `VirgeClient::with_xtransport()`
  - [ ] 正确初始化 `XTransportHandler`
  - [ ] 传递配置参数
- [ ] 改进 `connect()`
  - [ ] 传递 server_cid 和 server_port 到底层
  - [ ] 处理超时
  - [ ] 添加重连逻辑（可选）
- [ ] 增强 `send()` 和 `recv()`
  - [ ] 添加数据大小限制
  - [ ] 处理部分读写
- [ ] 添加支持多种传输协议的工厂方法
- [ ] 完善日志

### 4.2 服务器完善

**优先级**: 中

**文件**: `src/server/mod.rs`

**任务列表**:
- [ ] 完善 `VirgeServer::with_yamux()`
- [ ] 完善 `VirgeServer::with_xtransport()`
- [ ] 改进 `listen()`
  - [ ] 创建 vsock 监听器
  - [ ] 处理来自客户端的连接
  - [ ] 管理连接池
- [ ] 实现连接管理
  - [ ] 跟踪活跃连接
  - [ ] 强制关闭连接
  - [ ] 连接数限制
- [ ] 改进 `send()` 和 `recv()`
  - [ ] 支持多个客户端连接
  - [ ] 选择要通信的客户端
- [ ] 完善日志

## Phase 5: 测试

### 5.1 单元测试

**优先级**: 高

- [ ] 创建 `tests/unit/` 目录
- [ ] 编写错误类型测试
- [ ] 编写连接层测试
  - [ ] 测试连接/断开
  - [ ] 测试读写操作
  - [ ] 测试错误处理
- [ ] 编写传输层测试
  - [ ] 测试 Yamux 基本操作
  - [ ] 测试 XTransport 基本操作
  - [ ] 测试错误处理
- [ ] 编写应用层测试
  - [ ] 客户端连接测试
  - [ ] 服务器监听测试
  - [ ] 数据收发测试

### 5.2 集成测试

**优先级**: 高

- [ ] 创建 `tests/integration/` 目录
- [ ] 编写端到端测试
  - [ ] Yamux 客户端-服务器通信
  - [ ] XTransport 客户端-服务器通信
  - [ ] 双向通信测试
  - [ ] 多客户端场景
- [ ] 性能测试
  - [ ] 吞吐量测试
  - [ ] 延迟测试
  - [ ] 压力测试

### 5.3 示例代码

**优先级**: 中

- [ ] 创建 `examples/` 目录
- [ ] `examples/client_yamux.rs` - Yamux 客户端示例
- [ ] `examples/server_yamux.rs` - Yamux 服务器示例
- [ ] `examples/client_xtransport.rs` - XTransport 客户端示例
- [ ] `examples/server_xtransport.rs` - XTransport 服务器示例
- [ ] `examples/multi_client.rs` - 多客户端示例

## Phase 6: 文档与优化

### 6.1 文档

**优先级**: 中

- [ ] 完善 `README.md`
  - [ ] 项目介绍
  - [ ] 快速开始
  - [ ] 特征说明
  - [ ] 性能数据
- [ ] 生成 API 文档
  ```bash
  cargo doc --no-deps --open
  ```
- [ ] 编写性能指南
- [ ] 编写故障排除指南
- [ ] 编写扩展指南（添加新协议）

### 6.2 优化

**优先级**: 低

- [ ] 性能优化
  - [ ] 缓冲管理
  - [ ] 内存池
  - [ ] 锁竞争优化
- [ ] 添加连接池
- [ ] 添加自动重连机制
- [ ] 添加健康检查
- [ ] 添加监控指标

## 工作流程建议

1. **当前进度**: Phase 1 ✅ 完成
2. **下一步**: Phase 2 - 实现底层连接
   - 从 `TokioVsockImpl` 开始
   - 编写单元测试
   - 验证与 tokio-vsock 的集成
3. **然后**: Phase 3 - 实现传输协议
   - 同时进行 Yamux 和 XTransport 实现
   - 编写单元测试
   - 验证与各自库的集成
4. **后续**: Phase 4 - 完善应用层
   - 集成底层实现
   - 编写端到端测试
5. **最后**: Phase 5 & 6 - 测试和文档

## 编码标准

- [ ] 所有 public 类型需要 doc comments
- [ ] 所有 public 函数需要示例代码
- [ ] 使用 `log::info!`/`debug!`/`warn!`/`error!` 记录日志
- [ ] 错误优先使用 `?` 操作符
- [ ] 代码通过 `cargo clippy`
- [ ] 代码通过 `cargo fmt`
- [ ] 单元测试覆盖率 > 80%

## 编译和测试命令

```bash
# 编译（无特征）
cargo build --no-default-features

# 编译（Yamux）
cargo build --no-default-features --features "use-yamux"

# 编译（XTransport）
cargo build --no-default-features --features "use-xtransport"

# 编译（两者都启用）
cargo build --no-default-features --features "use-yamux use-xtransport"

# 运行所有测试
cargo test --no-default-features --features "use-yamux use-xtransport"

# 生成文档
cargo doc --no-deps --open

# 代码检查
cargo clippy --all-targets --no-default-features --features "use-yamux use-xtransport"

# 格式检查
cargo fmt --check

# 运行特定示例
cargo run --example client_yamux
```

## 联系与讨论

在实现过程中如有疑问，建议先：
1. 查阅架构文档 (`ARCHITECTURE.md`)
2. 查阅使用示例 (`EXAMPLES.md`)
3. 查看对应库的文档（yamux、xtransport、tokio-vsock）
4. 查看代码中的 TODO 注释
