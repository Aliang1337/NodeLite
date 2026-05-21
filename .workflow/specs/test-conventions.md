---
title: "Test Conventions"
readMode: required
priority: high
category: test
keywords:
  - test
  - coverage
  - mock
  - fixture
  - assertion
  - framework
---

# Test Conventions

Auto-generated from project analysis. Update manually as patterns evolve.

## Framework
- Framework: **内置 `cargo test`**（Rust 标准 `#[test]` / `#[tokio::test]`）
- Coverage: `cargo-tarpaulin`（项目根 `tarpaulin.toml` 配置）
- CI: GitHub Actions（`.github/workflows/ci.yml`、`coverage.yml`）
- 异步测试：`#[tokio::test]`（必要时 `flavor = "multi_thread"`）

## Directory Structure
- **单元测试**：与源文件同目录
  - 同文件内 `#[cfg(test)] mod tests { ... }`
  - 大模块拆出 `<module>/tests.rs`（如 `registry/tests.rs`、`config/tests.rs`）
  - 库级测试集中文件：`src/lib_tests.rs`
- **集成测试**：`<crate>/tests/integration/`
  - 入口：`tests/integration/mod.rs`
  - 主题文件：`token_lifecycle.rs`、`server_agent_handshake.rs`、`shutdown_signal.rs`、`metrics_collection.rs`、`failure_recovery.rs`、`concurrent_nodes.rs`
- **负载/场景测试**：`src/load_test/`（`scenarios.rs`、`fake_agent.rs`、`probes.rs`、`server.rs`）
- **测试辅助**：`src/test_support.rs` 提供公共 fixture / builder

## Naming Conventions
- 测试函数：`test_<scope>_<scenario>` 或 `<scope>_<scenario>_<expectation>`（snake_case）
- 测试模块：`mod tests` 或 `mod <feature>_tests`
- Happy / Error 路径分开：`test_happy_path` / `test_error_handling`
- 回归测试：`test_regression_<issue_id>_<short_description>`

## Patterns

### 强制要求（来自 CLAUDE.md）
- 新功能必须有单元测试
- Bug 修复必须有回归测试
- 覆盖率目标 75%+
- 关键模块（`auth`、`registry`、`ws`）覆盖率 85%+

### 推荐做法
- 测试中可用 `.expect("clear error message")`，**但禁止 `.unwrap()`**
- 异步并发用例：构造小规模真实并发，避免 `sleep` 同步；使用 `tokio::time::pause()` 控制时间
- 公共 fixture 放 `test_support.rs`，避免跨测试拷贝
- 集成测试使用真实 SQLite（temp 文件 / `:memory:`），不 mock 数据访问层

### 运行
- 全量：`cargo test`
- 单测：`cargo test <test_name> -- --nocapture`
- 覆盖率：`cargo tarpaulin`（按 `tarpaulin.toml`）

## Entries

