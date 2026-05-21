---
title: "Learnings"
readMode: optional
priority: medium
category: learning
keywords:
  - bug
  - lesson
  - gotcha
  - learning
---

# Learnings

Bugs, gotchas, and lessons learned during development.
Add entries with: `/spec-add learning <description>`

## Entries

### L-001 — 密码强度规则需收口于单一函数（来自 `auth.rs::validate_password_strength` 注释 #92）
**Context**: 启动期（`READONLY_PASSWORD` 环境变量）与管理后台改密 API 早期各自实现密码校验，规则容易漂移。
**Lesson**: 复用同一 `validate_password_strength` 函数（返回 `Result<(), &'static str>`），两边都直接展示同一文案，不依赖 `anyhow::Error::to_string()` 格式化。
**Apply to**: 任何"多入口共享同一约束"的场景——抽提到一个返回静态字符串的纯函数，避免 Error 类型间接传递使文案漂移。

### L-002 — Auth 层不可反向依赖 `AppState`
**Context**: `auth.rs` 注释明确"这一层不直接持有 `AppState`，避免 main.rs 的总状态结构反过来产生循环依赖"。
**Lesson**: 通用基础设施层（auth / sanitize / encoding）应只接受所需字段引用，而非整个 `AppState`。
**Apply to**: 新增横切模块时，明确入参为最小集合；调用方在 handler 中拆 state。

