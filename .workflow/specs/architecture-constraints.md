---
title: "Architecture Constraints"
readMode: required
priority: high
category: arch
keywords:
  - architecture
  - module
  - layer
  - boundary
  - dependency
  - structure
---

# Architecture Constraints

Auto-generated from project structure (Cargo workspace, 3 crates). Update manually as architecture evolves.

## Module Structure
- Type: **Monorepo / Cargo workspace**（`resolver = "3"`，edition = "2024"）
- Members:
  - `nodelite-proto/` — 协议定义与共享类型（消息、配置、模型、校验、网络工具）
  - `nodelite-agent/` — 节点端二进制：系统指标采集 → WebSocket 上报
  - `nodelite-server/` — 服务端二进制：接收、注册、持久化、Web UI

## Layer Boundaries

```
nodelite-proto  (无业务依赖，被两侧共享)
       │
       ▼
nodelite-agent ── WebSocket ──▶ nodelite-server
                                    │
                                    ├── Registry (内存)
                                    ├── History  (SQLite)
                                    └── Web UI    (实时展示)
```

服务端内部分层（`nodelite-server/src/`）：
- 入口：`main.rs` → `lib.rs::cli_main` → `startup.rs`
- 状态：`app_state.rs` / `state.rs`
- 接入：`ws.rs`（WebSocket）/ `ui.rs`（HTTP handlers）
- 业务：`registry.rs`（节点）/ `history.rs`（持久化）/ `auth.rs`（认证）/ `admission.rs`（准入）
- 横切：`sanitize.rs`（校验）/ `audit.rs`（审计）/ `encoding.rs`、`qr.rs`、`fs_security.rs`
- 后台：`background.rs`
- 测试支持：`test_support.rs` / `lib_tests.rs` / `load_test/`

## Dependency Rules
- `nodelite-proto` 必须保持纯类型与协议，**禁止依赖 server/agent 内部模块**
- `nodelite-agent` 与 `nodelite-server` **不可互相依赖**，仅通过 `nodelite-proto` + WebSocket 通信
- `handlers/ui` 层不可直接修改 `AppState` 内部不变量；状态变更走 `registry` / `history` 等业务模块
- 后台任务（`background.rs`）只读取 / 改写已封装的并发原语，不引入新的全局锁

## Technology Constraints
- Runtime: Rust edition **2024**，Tokio（features = `full`）
- TLS: **rustls + ring**（禁止 OpenSSL；通过 `default-features = false` 关闭其它后端）
- DB: SQLite via `rusqlite`（features = `bundled`，避免系统 libsqlite 依赖）
- WebSocket: `axum` 0.8（features = `ws`）+ `tokio-tungstenite`
- 序列化: `serde` + `serde_json` + `toml`/`toml_edit`
- Build profile（release）: `codegen-units = 1`、`lto = "thin"`、`opt-level = 3`、`strip = "symbols"`
- 二进制目标尺寸约束：服务端 4-10MB、Agent 800KB
- 内存约束：服务端常驻 < 15MB，Agent < 2MB

## Entries

