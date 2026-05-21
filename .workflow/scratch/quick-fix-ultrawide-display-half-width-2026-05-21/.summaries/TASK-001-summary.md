# TASK-001 Summary — 修复带鱼屏显示器界面展示一半的问题

**完成时间**: 2026-05-21T09:02:28Z
**状态**: completed
**改动文件**: 2 个

## 根因

`nodelite-server/assets/index.html` 与 `node.html` 的主内容容器 `.main` 同时设置了：

```css
.main {
  max-width: 1680px;
  width: 100%;
  /* 没有 margin: 0 auto;,也没有 min-width 媒体查询 */
}
```

而 `.app` 是 grid：`grid-template-columns: 72px minmax(0, 1fr);`。在带鱼屏（21:9 ≈ 2560×1080、32:9 ≈ 3840×1080 / 5120×1440）下：

1. `.main` 所在的 grid cell 宽度远大于 1680px
2. 元素受 `max-width: 1680px` 限制，实际只渲染 1680px 宽
3. grid item 默认 `justify-self: stretch` 在子元素有显式宽度时退化为 `start`，导致 `.main` **左对齐**到 sidebar 右侧
4. 右侧出现大量空白，看起来"界面只展示一半"

## 改动

### 1. `nodelite-server/assets/index.html`

**L171-178** — 给 `.main` 增加居中：

```css
.main {
  padding: 24px clamp(20px, 3vw, 36px) 40px;
  max-width: 1680px;
  width: 100%;
  /* 在 grid cell 内居中,避免带鱼屏(21:9 / 32:9)下被左对齐到 sidebar 旁边 */
  margin-left: auto;
  margin-right: auto;
}
```

**L819-826** — 新增超宽屏响应式断点：

```css
@media (min-width: 1920px) {
  .main { max-width: 2200px; }
}
@media (min-width: 2560px) {
  .main { max-width: 2400px; }
}
```

### 2. `nodelite-server/assets/node.html`

同步相同改动：
- L171-178 `.main` 增加 `margin-left/right: auto;`
- L1055-1062 末尾追加 `@media (min-width: 1920px)` 与 `@media (min-width: 2560px)` 断点

## 设计决策

| 决策 | 理由 |
|------|------|
| 保留默认 `max-width: 1680px` | 不破坏既有 < 1920px 桌面显示器体验 |
| 1920px 断点 → 2200px | 常见 2560×1440 / 3440×1440 屏幕，保留约 250-600px 视觉留白避免内容贴边 |
| 2560px 断点 → 2400px | 超宽屏（5120×1440、32:9）下进一步放宽，但仍留 ~160px 边距维持阅读舒适度 |
| 使用 `margin-left/right: auto`（而非简写 `margin: 0 auto`） | 避免覆盖原有 `padding` 上下值；最小侵入式改动 |
| 仅 CSS，不引入 JS | 符合既有 inline `<style>` + CSP 模型，无新依赖 |

## 验收（Convergence Criteria）

- ✅ index.html / node.html `.main` 选择器均添加 `margin-left: auto; margin-right: auto;`
- ✅ 两文件均新增 `@media (min-width: 1920px) { .main { max-width: 2200px; } }`
- ✅ 两文件均新增 `@media (min-width: 2560px) { .main { max-width: 2400px; } }`
- ✅ 原 `max-width: 1680px` 保留为默认
- ✅ 现有 `max-width: 1320/900/720` 断点未受影响
- ✅ 无 JS 改动
- ✅ `cargo build -p nodelite-server` 通过（`include_str!` 嵌入资源编译成功）

## 测试建议

在 nodelite-server 启动后，在以下分辨率/缩放下检查 `/` 与 `/node?id=<x>` 页面：

| 分辨率 | 预期行为 |
|--------|---------|
| 1280×720 | 与之前完全一致（`max-width: 1320` 媒体查询激活） |
| 1920×1080 | 进入 `min-width: 1920px` 断点，`.main` 提升至 2200px（仍受窗口限制） |
| 2560×1080（21:9 带鱼屏） | `.main` ≈ 2200px 居中，左右留白对称（之前是单侧空白） |
| 3440×1440 | `.main` 2200px 居中 |
| 5120×1440（32:9） | `.main` 2400px 居中，两侧约 1360px 留白 |

## 影响范围

- 仅前端样式，无 API / 协议 / 持久化变更
- 不破坏向后兼容性
- 二进制大小变化：两文件分别新增约 350 字节 CSS → release strip 后接近零额外占用
