# Virga Example 测试程序

本目录包含 virga 库的测试示例程序。

## 目录结构

```
example/
├── Cargo.toml          # workspace 配置，默认使用 xtransport 特征
├── client_test/        # 客户端测试程序
│   └── src/main.rs
└── server_test/        # 服务端测试程序
    └── src/main.rs
```

## 协议选择

默认使用 `use-xtransport` 特征。如需切换到 `yamux` 协议，修改 `Cargo.toml`：

```toml
[workspace.dependencies]
virga = { path = "/home/kylin/code/virga", default-features = false, features = ["use-yamux"]}
```

## 测试用例说明

### test_1 - 基本协议测试
测试带长度前缀的数据传输：
- 客户端：先发送数据长度（8字节），再发送数据
- 服务端：先接收长度，再接收数据，然后回显

### test_2 - 分块读取测试
测试小缓冲区循环读取：
- 客户端：发送 512 字节数据
- 使用 8 字节小缓冲区循环读取
- 验证总接收长度正确

### test_3 - 简单回显测试
测试基本的 send/recv API：
- 客户端：发送 512 字节数据
- 服务端：接收后回显
- 验证数据一致性

### test_4 - 性能测试 ⭐
测试大数据量传输性能：
- **数据大小**: 10 MB
- **测试次数**: 10 次迭代
- **测试内容**: 
  - 客户端发送 10MB 数据
  - 服务端接收并回显
  - 客户端接收回显数据
- **输出指标**:
  - 平均发送/接收时间
  - 总数据量和总耗时
  - 平均吞吐量 (MB/s)
  - 发送速度和接收速度

## 运行测试

### 1. 启动服务端

```bash
cd example
cargo run --bin server_test
```

### 2. 启动客户端

在另一个终端：

```bash
cd example
cargo run --bin client_test
```

### 3. 切换测试用例

在 `main.rs` 中注释/取消注释对应的测试函数：

**客户端 (client_test/src/main.rs)**:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
    
    // test_1(&mut client)?;  // 协议测试
    // test_2(&mut client)?;  // 分块读取
    // test_3(&mut client)?;  // 简单回显
    test_4(&mut client)?;     // 性能测试 ← 当前激活
    
    // ...
}
```

**服务端 (server_test/src/main.rs)**:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
    
    // test_1(&mut server)?;  // 协议测试
    // test_2(&mut server)?;  // 分块读取
    // test_3(&mut server)?;  // 简单回显
    test_4(&mut server)?;     // 性能测试 ← 当前激活
    
    // ...
}
```

## 性能测试输出示例

### 客户端输出:
```
=== 性能测试开始 ===
数据大小: 10 MB
测试次数: 10
第 1 次 - 发送: 45.23 ms, 接收: 42.18 ms
第 2 次 - 发送: 43.56 ms, 接收: 41.32 ms
...
第 10 次 - 发送: 44.01 ms, 接收: 40.98 ms

=== 性能测试结果 ===
平均发送时间: 44.12 ms
平均接收时间: 41.50 ms
总数据量: 200.00 MB
总耗时: 0.86 秒
平均吞吐量: 233.26 MB/s
发送速度: 226.76 MB/s
接收速度: 241.00 MB/s
```

### 服务端输出:
```
=== 服务端性能测试开始 ===
测试次数: 10
第 1 次 - 接收了 10485760 bytes (10.00 MB) in 42.15 ms
第 1 次 - 发送了 10485760 bytes (10.00 MB) in 45.20 ms
...

=== 服务端性能测试结果 ===
平均接收时间: 41.48 ms
平均发送时间: 44.18 ms
总数据量: 200.00 MB
总耗时: 0.86 秒
平均吞吐量: 233.72 MB/s
接收速度: 241.16 MB/s
接收速度: 226.46 MB/s
```

## 日志级别

可通过环境变量 `RUST_LOG` 调整日志级别：

```bash
# 详细调试日志
RUST_LOG=debug cargo run --bin client_test

# 仅信息日志（默认）
RUST_LOG=info cargo run --bin client_test

# 警告及以上
RUST_LOG=warn cargo run --bin client_test
```

## 注意事项

1. **服务端需要先启动**，客户端才能连接
2. **测试用例要匹配**：客户端和服务端需要运行相同的 test 函数
3. **性能测试建议**：
   - 使用 release 模式获得更准确的性能数据：`cargo run --release --bin client_test`
   - 确保网络环境稳定
   - 多次运行取平均值
4. **协议切换**：修改 `Cargo.toml` 后需要重新编译
