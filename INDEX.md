# 📑 Virga 项目导航索引

欢迎来到 Virga 项目！本文档帮助你快速找到所需的信息。

---

## 🎯 我想...

### ...快速了解项目
1. **第一步（5 分钟）**：阅读 `QUICK_REFERENCE.md`
   - 项目结构、核心概念、编译命令速查
2. **第二步（10 分钟）**：查看本项目的 README（待编写）

### ...理解整个架构设计
1. **阅读**：`ARCHITECTURE.md`（强烈推荐！）
   - 分层架构、模块职责、数据流、扩展点
   - 大约需要 15-20 分钟

### ...学习如何使用 API
1. **阅读**：`EXAMPLES.md`
   - 客户端/服务器使用示例
   - 特征使用示例
   - 错误处理示例
   - 自定义协议实现

### ...开始开发/实现功能
1. **查看**：`TODO.md`
   - Phase 1-6 详细任务清单
   - 每个 Phase 的实现指导
   - 优先级和依赖关系
   - 编码标准

### ...找到某个模块的代码
参考源代码树：
```
src/
├── error/mod.rs           # 错误定义
├── connection/mod.rs      # 连接 trait
├── transport/mod.rs       # 协议 trait
│   ├── yamux_impl/
│   └── xtransport_impl/
├── client/mod.rs          # 客户端
└── server/mod.rs          # 服务器
```

### ...查找 API 文档
```bash
cargo doc --no-deps --open
```
或直接查看源代码的 doc comments。

### ...快速命令参考
见 `QUICK_REFERENCE.md` 中的"编译命令速查"。

### ...实现一个新的传输协议
1. 阅读 `ARCHITECTURE.md` 中的"扩展点"
2. 参考 `EXAMPLES.md` 中的"实现自定义传输协议"
3. 按 `TODO.md` Phase 3 的模板实现

### ...检查项目编译状态
```bash
cargo build --no-default-features --features "use-yamux use-xtransport"
```

---

## 📚 文档结构

```
.
├── 🚀 快速入门（推荐阅读顺序）
│   ├── QUICK_REFERENCE.md      ← 先读这个（5 min）
│   ├── ARCHITECTURE.md         ← 再读这个（20 min）
│   └── EXAMPLES.md             ← 然后读这个（10 min）
│
├── 📖 深入学习
│   ├── DESIGN_SUMMARY.md       ← 完成工作总结
│   ├── TODO.md                 ← 开发清单和实现指导
│   └── 源代码注释               ← 模块详细文档
│
├── 📑 本文档
│   └── INDEX.md（你在这里）    ← 导航和索引
│
└── ❓ 其他
    └── README.md（待编写）      ← 项目介绍
```

---

## 🔍 关键概念快速查找

| 概念 | 文档位置 | 说明 |
|------|---------|------|
| 4 层分层架构 | ARCHITECTURE.md 2.2 | 应用层→协议层→连接层→错误层 |
| Transport trait | transport/mod.rs | 统一传输协议接口 |
| VsockConnection trait | connection/mod.rs | 统一连接接口 |
| VirgeClient API | client/mod.rs | 客户端使用接口 |
| VirgeServer API | server/mod.rs | 服务器使用接口 |
| Feature 管理 | Cargo.toml | use-yamux、use-xtransport 特征 |
| 错误处理 | error/mod.rs | VirgeError 类型定义 |
| Yamux 实现 | transport/yamux_impl/mod.rs | 多路复用传输实现 |
| XTransport 实现 | transport/xtransport_impl/mod.rs | 轻量级传输实现 |

---

## 📊 项目状态速查

### 当前进度
- ✅ **Phase 1**：基础框架设计完成
- ⏳ **Phase 2**：底层实现（待开始）
- ⏳ **Phase 3**：协议实现（待开始）
- ⏳ **Phase 4**：应用层完善（待开始）
- ⏳ **Phase 5**：测试（待开始）
- ⏳ **Phase 6**：文档优化（待开始）

### 编译状态
- ✅ 框架编译成功
- ✅ 所有 trait 定义正确
- ✅ 所有模块结构完整
- ✅ 特征配置有效

---

## 🛠️ 常见任务快速指南

### 任务 1：编译项目

```bash
# 无特征（仅错误层）
cargo build --no-default-features

# Yamux 特征
cargo build --no-default-features --features "use-yamux"

# XTransport 特征
cargo build --no-default-features --features "use-xtransport"

# 两者都启用
cargo build --no-default-features --features "use-yamux use-xtransport"
```

### 任务 2：运行测试

```bash
# 所有特征
cargo test --no-default-features --features "use-yamux use-xtransport"

# 仅 Yamux
cargo test --no-default-features --features "use-yamux"
```

### 任务 3：代码检查

```bash
# Clippy 检查
cargo clippy --no-default-features --features "use-yamux use-xtransport"

# 格式检查
cargo fmt --check

# 完整检查
cargo fmt && cargo clippy --no-default-features --features "use-yamux use-xtransport"
```

### 任务 4：生成文档

```bash
# 本地查看
cargo doc --no-deps --open

# 仅构建
cargo doc --no-deps
```

### 任务 5：实现新功能

1. 阅读 `TODO.md` 找到对应 Phase 的任务
2. 查看源代码中的 `TODO:` 注释
3. 参考 `EXAMPLES.md` 和已有实现
4. 编写代码和测试
5. 运行 `cargo build` 和 `cargo test` 验证

---

## ❓ 常见问题

### Q1：从哪里开始？
**A**：
1. 先读 `QUICK_REFERENCE.md`（5 分钟）
2. 再读 `ARCHITECTURE.md`（20 分钟）
3. 然后看 `EXAMPLES.md`（10 分钟）
4. 开始按 `TODO.md` 实现

### Q2：项目当前完成度如何？
**A**：框架设计完成 100%，具体实现 0%。
- ✅ 所有 trait 定义完成
- ✅ 所有模块结构完成
- ✅ 文档完善完成
- ⏳ 具体实现（连接层、协议层等）待完成

### Q3：怎样添加新的传输协议？
**A**：
1. 阅读 `ARCHITECTURE.md` 中的"扩展点"
2. 在 `src/transport/` 创建新模块
3. 实现 `Transport` trait
4. 在 `Cargo.toml` 添加 feature
5. 在应用层添加工厂方法

### Q4：如何编译和测试？
**A**：见上面的"常见任务快速指南"。

### Q5：文档太多了，应该先读哪个？
**A**：按这个顺序：
1. `QUICK_REFERENCE.md`（5 min）
2. `ARCHITECTURE.md`（20 min）
3. `EXAMPLES.md`（10 min）
4. 开始编码，遇到问题再查 `TODO.md`

---

## 📞 获取帮助

| 问题类型 | 查看位置 |
|---------|---------|
| 架构相关 | ARCHITECTURE.md |
| 使用相关 | EXAMPLES.md |
| 实现相关 | TODO.md 中的实现指导 |
| API 相关 | 源代码 doc comments |
| 特征相关 | QUICK_REFERENCE.md |
| 命令相关 | QUICK_REFERENCE.md 中的"编译命令速查" |

---

## 📈 学习路径建议

### 初学者（1-2 天）
1. QUICK_REFERENCE.md（5 min）
2. ARCHITECTURE.md（20 min）
3. EXAMPLES.md（10 min）
4. 浏览源代码（30 min）
→ 理解整个架构和 API 设计

### 实现者（数天）
1. TODO.md - Phase 2（连接层实现）
2. 根据源代码中的 `TODO:` 注释实现
3. 编写单元测试
4. 验证编译和测试通过
→ 逐个 Phase 推进项目

### 维护者（持续）
1. DESIGN_SUMMARY.md - 了解设计决策
2. ARCHITECTURE.md - 理解架构原则
3. 按 TODO.md 的编码标准维护代码
4. 更新文档
→ 确保代码质量和文档准确

---

## 🔗 文件快速链接

```
项目根目录
├── src/
│   ├── lib.rs              → 公共 API 导出
│   ├── error/mod.rs        → 错误定义
│   ├── connection/mod.rs   → 连接 trait
│   ├── transport/mod.rs    → 协议 trait
│   ├── client/mod.rs       → 客户端 API
│   └── server/mod.rs       → 服务器 API
│
├── Cargo.toml              → 依赖和特征配置
├── Cargo.lock              → 依赖锁文件
│
└── 📄 文档
    ├── QUICK_REFERENCE.md  → 快速参考（⭐ 先读）
    ├── ARCHITECTURE.md     → 详细架构（⭐ 必读）
    ├── EXAMPLES.md         → 使用示例
    ├── TODO.md             → 开发清单
    ├── DESIGN_SUMMARY.md   → 完成总结
    └── INDEX.md            → 本文档
```

---

## ✨ 快速开始模板

复制粘贴即用：

### 编译和检查
```bash
cd /home/greatwall/code/virga
cargo build --no-default-features --features "use-yamux use-xtransport"
cargo clippy --no-default-features --features "use-yamux use-xtransport"
cargo fmt
```

### 查看文档
```bash
# 快速参考
less QUICK_REFERENCE.md

# 详细架构
less ARCHITECTURE.md

# 使用示例
less EXAMPLES.md

# 开发清单
less TODO.md

# 生成 API 文档
cargo doc --no-deps --open
```

### 修改代码后
```bash
# 格式化
cargo fmt

# 编译检查
cargo build --no-default-features --features "use-yamux use-xtransport"

# Lint 检查
cargo clippy --no-default-features --features "use-yamux use-xtransport"

# 运行测试
cargo test --no-default-features --features "use-yamux use-xtransport"
```

---

## 📝 更新记录

- **2026-01-15**：框架设计完成，所有文档编写完毕

---

**祝你开发愉快！** 🚀

如有问题，请参考对应的文档或查看源代码中的 TODO 注释。
