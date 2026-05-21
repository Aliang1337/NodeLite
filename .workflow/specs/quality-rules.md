---
title: "Quality Rules"
readMode: required
priority: medium
category: review
keywords:
  - quality
  - lint
  - rule
  - enforcement
---

# Quality Rules

Auto-derived from `CLAUDE.md` + CI workflows. Enforced via review and CI.

## Hard Rules（不可妥协）

| ID | 规则 | 检查方式 |
|----|------|---------|
| Q-001 | 生产代码禁用 `.unwrap()` / `.expect()` | review + grep（允许测试代码） |
| Q-002 | 所有 SQL 必须参数化（`params!` + `?N`） | review + grep `format!.*INSERT\|UPDATE\|SELECT` |
| Q-003 | 敏感比较必须使用 `subtle::ConstantTimeEq` | review |
| Q-004 | TLS 仅允许 `rustls`（禁止 `openssl-*`） | `cargo tree` 审计 |
| Q-005 | CSPRNG 必须使用 `getrandom`（禁止 `rand::thread_rng` 用于密钥/Token） | review |
| Q-006 | 敏感文件权限：文件 0600 / 目录 0700 | `fs_security.rs` 单测覆盖 |
| Q-007 | 禁止在日志中输出密码 / Token / TOTP secret | review |

## Soft Rules（强烈推荐）

| ID | 规则 | 说明 |
|----|------|------|
| Q-101 | 单文件 < 500 行（目标），< 800 行（硬上限） | `main.rs` 应 < 200 行 |
| Q-102 | 公开类型必须有 doc comment | `///` / `//!` |
| Q-103 | 引入新依赖需评审：维护活跃度、License、安全记录 | License 必须兼容 MIT / Apache-2.0 |
| Q-104 | Release profile 不可下调（`lto=thin`、`codegen-units=1`、`strip=symbols`） | 影响最终二进制尺寸 |

## CI Gates

- `.github/workflows/ci.yml` — 构建 + 测试
- `.github/workflows/coverage.yml` — tarpaulin 覆盖率
- `.github/workflows/release.yml` — 发布流程

## Entries

