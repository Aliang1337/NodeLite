# Project: NodeLite

## What This Is

NodeLite 是一款采用 Server-Agent 架构的轻量级 Rust 监控系统，面向资源受限或追求最小开销的部署环境。通过 WebSocket 实时上报系统指标，结合 Token + 可选 TOTP 2FA 认证与 SQLite 历史存储，为运维者提供低延迟、低占用的节点可观测能力。

## Core Value

**极低资源占用的轻量监控** — 服务端常驻 <15MB、Agent <2MB；这是产品的根本身份。在任何取舍中，资源占用与运行时开销的底线优先于功能扩展。

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- WebSocket 实时双向通信（`nodelite-server/src/ws.rs`）
- Token 认证 + 可选 TOTP 2FA（`auth.rs`）
- SQLite 历史数据持久化（`history.rs`）
- 节点注册与会话管理（`registry.rs`）
- 输入验证与清洗（`sanitize.rs`）
- 限流准入控制（`admission.rs`）
- 共享状态并发管理（`Arc<RwLock<T>>` + `AtomicU64`）
- 跨平台 Agent 系统指标采集（`nodelite-agent/src/collector.rs`）
- 已通过 200 节点 / 18,677 指标/秒 / p95 < 5ms 基准

### Active

<!-- Current scope being built toward. These are hypotheses until shipped. -->

当前阶段重心：**性能优化与可靠性**

- [ ] 错误恢复增强：消除生产路径残留 `.unwrap()` / `.expect()`，统一 `anyhow::Context` 边界
- [ ] 写入路径吞吐优化：SQLite 批量事务、连接池复用、热路径零拷贝评估
- [ ] WebSocket 健壮性：断线重连、心跳超时、背压处理的端到端测试覆盖
- [ ] 大文件拆分以利于演进：`main.rs` (1674 行) 与 `registry.rs` (1439 行) 模块化重构
- [ ] 关键模块测试覆盖率达成 85%+（auth、registry、ws）

### Out of Scope

<!-- Explicit boundaries. Include reasoning to prevent re-adding. -->

- 重型告警/可视化平台 — 与"轻量"定位冲突，由下游接入方处理
- 多租户 / SaaS 化部署 — 单体监控定位，不引入鉴权域、租户隔离等复杂度
- 自研可视化前端框架 — 维持简洁内置 Web UI，复杂展示交由 Grafana 等已有生态
- 替换 SQLite 为分布式数据库 — 资源占用红线不允许引入外部依赖
- OpenSSL — 已选定 rustls，避免 C 依赖与编译复杂度

## Context

- **当前规模**：Rust workspace 由 3 个 crate 组成（`nodelite-proto`、`nodelite-agent`、`nodelite-server`），共计若干核心模块约 5000+ 行业务代码
- **代码质量现状**：核心模块基本稳定，但 `main.rs` 与 `registry.rs` 超出 800 行最大限，演进时存在认知与冲突成本
- **既有约束（来自 CLAUDE.md）**：生产代码禁用 `.unwrap()` / `.expect()`，所有 SQL 必须参数化，敏感比较必须常量时间，敏感文件 0600/0700 权限
- **测试基线**：覆盖率目标 75%+，关键模块 85%+
- **依赖原则**：最小化、版本固定、许可证兼容 MIT/Apache-2.0

## Constraints

- **Performance**：服务端常驻 < 15 MB，Agent < 2 MB — 核心价值底线
- **Latency**：在 200 节点并发下保持 p95 < 5ms — 当前已达成的基准不可回退
- **Security**：所有外部输入经 `sanitize.rs` 校验；密码/Token 比较使用 `subtle` crate 常量时间实现
- **Safety**：禁止 `.unwrap()` / `.expect()` 出现于生产路径；CSPRNG 使用 `getrandom`
- **Stack**：TLS 必须使用 `rustls`，禁止引入 OpenSSL；异步运行时锁定 Tokio
- **Backward Compatibility**：已发布的 WebSocket 协议与 HTTP API 不得破坏式变更
- **File Size**：单文件目标 < 500 行，硬上限 800 行，`main.rs` 应 < 200 行

## Tech Stack

- **Language**: Rust (edition 2024)
- **Async Runtime**: Tokio
- **Web Framework**: Axum 0.8（含 WebSocket 支持）
- **Database**: SQLite（通过 `rusqlite`，单文件本地存储）
- **Serialization**: serde / serde_json
- **TLS**: rustls（不使用 OpenSSL）
- **Crypto / Security**: argon2、subtle、getrandom
- **Logging**: tracing
- **CLI**: clap 4.5

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 采用 Rust + Tokio + Axum 技术栈 | 内存安全、零运行时、单二进制部署，契合极低资源占用目标 | ✅ 已验证（服务端 4-10MB） |
| 使用 SQLite 作为历史存储 | 嵌入式、零运维、单文件可移植，规避外部依赖 | ✅ 已验证 |
| TLS 选用 rustls 而非 OpenSSL | 纯 Rust 实现，规避 C 依赖与跨平台编译复杂度 | ✅ 已验证 |
| Token + 可选 TOTP 2FA 的认证策略 | 兼顾轻量自托管与生产级安全可选项 | ✅ 已验证 |
| 共享状态采用 `Arc<RwLock<T>>` + `AtomicU64` | 读多写少场景下平衡并发性能与代码简洁性 | ✅ 已验证 |
| 当前阶段聚焦"性能优化与可靠性" | 功能已基本稳定，下一阶段瓶颈在韧性与可演进性 | — Pending |

## Stakeholders

- **维护者**：NodeLite Contributors（开源协作模式）
- **用户群**：自托管运维者、嵌入式 / 边缘环境用户、追求最小依赖的小型团队
- **上游生态**：Rust 异步生态（tokio-rs、axum 等）
- **许可证**：MIT

---
*Last updated: 2026-05-21 after initialization*
