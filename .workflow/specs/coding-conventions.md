---
title: "Coding Conventions"
readMode: required
priority: high
category: coding
keywords:
  - style
  - naming
  - import
  - pattern
  - convention
  - formatting
---

# Coding Conventions

Auto-generated from project analysis (Rust workspace, edition 2024). Update manually as patterns evolve.

## Formatting
- Indentation: 4 spaces（Rust 标准；项目未提供 `rustfmt.toml`，默认 `rustfmt` 规则）
- Line length: 默认（rustfmt 默认 100）
- Trailing commas: 多行字面量保留尾随逗号（rustfmt 默认）
- 文件末尾保留换行

## Naming
- Variables/functions: `snake_case`（Rust 标准）
- Structs / Enums / Traits / 类型别名: `PascalCase`
- 常量与静态: `SCREAMING_SNAKE_CASE`
- 模块/文件: `snake_case.rs`
- Crate 名: `kebab-case`（`nodelite-server`、`nodelite-agent`、`nodelite-proto`）

## Imports
- 顺序：`std` → 外部 crate → 本地 crate（`crate::` / `super::`）
- 分组之间空行分隔
- 多导入合并：使用 `use foo::{bar, baz}`
- 不使用全局 `use *`，避免命名空间污染

## Patterns

### 错误处理（强制）
- 生产代码禁止 `.unwrap()` / `.expect()`；使用 `?` 传播
- 错误上下文使用 `anyhow::Context`：`.context("Failed to ...")`
- 公开 API 边界自定义错误类型（如 `AuthSessionError`），避免裸 `anyhow::Error` 扩散至 handler
- 测试代码允许 `.expect("clear message")`

### 并发与共享状态
- 共享只读为主：`Arc<RwLock<T>>`
- 共享互斥：`Arc<Mutex<T>>`（`auth.rs` 的 `TwoFactorSessions`）
- 无锁计数：`AtomicU64`（如 `next_session_id`）
- 异步任务：`tokio::spawn`
- RAII 资源管理（如 `WsConnectionPermit`）

### 安全模式（强制）
- 敏感比较：`subtle::ConstantTimeEq::ct_eq`
- 随机数：`getrandom::fill`（CSPRNG），禁止其它 RNG
- SQL：必须参数化（`params!` 宏 + `?N` 占位）
- 外部输入：经 `sanitize.rs` 校验后再使用
- 敏感文件权限：0600；目录 0700

### 日志
- 使用 `tracing`（`tracing::warn!` / `tracing::info!` 等）
- 禁止在日志中输出密码 / Token / TOTP secret 明文

### 模块组织
- 每个文件单一职责，目标 < 500 行，硬上限 800 行
- `main.rs` 应保持极薄：仅 `#[tokio::main]` 入口委托 `lib.rs::cli_main`
- 大模块可下钻为子目录（如 `registry/` 下含 `mod.rs`、`error.rs`、`tests.rs`、`load_test/`）

### 注释
- 模块/公开 API 使用 `//!` / `///` doc comment
- 项目注释语言：**中文**（与既有代码库一致）
- 行内注释聚焦"为什么"，不解释"做什么"

## Entries

