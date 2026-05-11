// 前端 UI 资源:
// - `index_html` / `node_html` 把 HTML 模板和注入参数拼接后返回给浏览器;
// - 模板里嵌入的 CSS + JavaScript 在编译期不做加工,运行期由浏览器执行;
// - `INDEX_TEMPLATE` 由 `assets/index.html` 提供,便于在编辑器里维护完整的视图;
//   暂未抽出的 `NODE_TEMPLATE` 仍内嵌为 Rust 原始字符串;
// - 国际化字典放在 `assets/ui-i18n.json`,通过 `include_str!` 一并编译进二进制。

/// 编译期嵌入的前端 i18n 字典,前端通过 `/assets/ui-i18n.json` 拉取。
pub const UI_I18N_JSON: &str = include_str!("../assets/ui-i18n.json");
/// 前端 i18n 字典对应的 HTTP 路径,统一注入到模板中。
pub const UI_I18N_ASSET_PATH: &str = "/assets/ui-i18n.json";

/// 渲染首页 HTML:把刷新间隔与 i18n 资源路径替换到模板占位符里。
pub fn index_html(refresh_interval_secs: u64) -> String {
    INDEX_TEMPLATE
        .replace(
            "__REFRESH_MS__",
            &(refresh_interval_secs * 1000).to_string(),
        )
        .replace("__I18N_ASSET_PATH__", UI_I18N_ASSET_PATH)
}

/// 渲染节点详情页 HTML;额外把当前节点 ID 以 JSON 编码后嵌入模板,避免 XSS。
pub fn node_html(node_id: &str, refresh_interval_secs: u64) -> String {
    NODE_TEMPLATE
        .replace(
            "__REFRESH_MS__",
            &(refresh_interval_secs * 1000).to_string(),
        )
        .replace("__I18N_ASSET_PATH__", UI_I18N_ASSET_PATH)
        .replace(
            "__NODE_ID_JSON__",
            &serde_json::to_string(node_id).unwrap_or_else(|_| "\"\"".to_string()),
        )
}

const INDEX_TEMPLATE: &str = include_str!("../assets/index.html");

const NODE_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>XiMonitor Node</title>
    <style>
      :root {
        color-scheme: light;
        --bg: #f7f2e9;
        --ink: #1a202b;
        --muted: #5d6875;
        --line: rgba(26, 32, 43, 0.1);
        --panel: rgba(255, 255, 255, 0.87);
        --accent: #0f766e;
        --chart-a: #0f766e;
        --chart-b: #b45309;
        --chart-c: #1d4ed8;
        --chart-d: #be185d;
        font-family: "Avenir Next", "Segoe UI", sans-serif;
      }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        min-height: 100vh;
        color: var(--ink);
        background:
          radial-gradient(circle at top left, rgba(208, 228, 227, 0.9), transparent 30%),
          radial-gradient(circle at top right, rgba(250, 228, 195, 0.6), transparent 24%),
          linear-gradient(135deg, var(--bg), #eef1f2);
      }
      .shell {
        width: min(1280px, calc(100vw - 32px));
        margin: 0 auto;
        padding: 24px 0 48px;
      }
      a { color: inherit; }
      .topline {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 18px;
        margin-bottom: 18px;
      }
      .topline .back {
        text-decoration: none;
        color: var(--muted);
        font-weight: 600;
      }
      .topline-actions {
        display: flex;
        align-items: center;
        gap: 14px;
      }
      .lang-picker {
        display: inline-flex;
        align-items: center;
        gap: 10px;
        color: var(--muted);
        font-size: 0.92rem;
      }
      .lang-select {
        border: 1px solid rgba(26, 32, 43, 0.12);
        border-radius: 999px;
        padding: 10px 14px;
        background: rgba(255, 255, 255, 0.82);
        color: var(--ink);
        font: inherit;
      }
      .hero, .panel {
        background: var(--panel);
        border: 1px solid var(--line);
        border-radius: 24px;
        box-shadow: 0 18px 60px rgba(26, 32, 43, 0.08);
        backdrop-filter: blur(18px);
      }
      .hero {
        padding: 24px;
        margin-bottom: 18px;
      }
      .hero h1 {
        margin: 0;
        font: 700 clamp(2.4rem, 4.8vw, 4.1rem) / 0.92 "Iowan Old Style", "Palatino Linotype", serif;
        letter-spacing: -0.05em;
      }
      .meta {
        margin-top: 10px;
        color: var(--muted);
        line-height: 1.7;
      }
      .stats, .charts {
        display: grid;
        gap: 16px;
      }
      .stats {
        grid-template-columns: repeat(4, minmax(0, 1fr));
        margin-bottom: 18px;
      }
      .panel {
        padding: 18px 20px;
      }
      .label {
        color: var(--muted);
        text-transform: uppercase;
        letter-spacing: 0.08em;
        font-size: 0.84rem;
      }
      .value {
        margin-top: 8px;
        font-size: clamp(1.5rem, 2.7vw, 2.2rem);
        font-weight: 700;
      }
      .controls-panel {
        margin-bottom: 18px;
      }
      .controls-head {
        display: flex;
        justify-content: space-between;
        gap: 18px;
        align-items: start;
      }
      .control-value {
        margin-top: 8px;
        font-size: 1.28rem;
        font-weight: 700;
        letter-spacing: -0.03em;
      }
      .control-subtle {
        margin-top: 8px;
        color: var(--muted);
        font-size: 0.92rem;
      }
      .controls-actions {
        display: flex;
        align-items: center;
        gap: 12px;
        flex-wrap: wrap;
        justify-content: end;
      }
      .preset-group {
        display: inline-flex;
        border: 1px solid rgba(26, 32, 43, 0.12);
        border-radius: 999px;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.7);
      }
      .preset-button {
        border: 0;
        background: transparent;
        color: var(--muted);
        padding: 11px 16px;
        font: inherit;
        font-weight: 700;
        cursor: pointer;
        transition: background 160ms ease, color 160ms ease;
      }
      .preset-button + .preset-button {
        border-left: 1px solid rgba(26, 32, 43, 0.08);
      }
      .preset-button.active {
        background: rgba(15, 118, 110, 0.14);
        color: var(--accent);
      }
      .toggle-button {
        border: 1px solid rgba(26, 32, 43, 0.12);
        border-radius: 999px;
        background: rgba(255, 255, 255, 0.75);
        color: var(--ink);
        padding: 12px 16px;
        font: inherit;
        font-weight: 700;
        cursor: pointer;
        transition: background 160ms ease, color 160ms ease, border-color 160ms ease;
      }
      .toggle-button.active {
        background: rgba(15, 118, 110, 0.12);
        border-color: rgba(15, 118, 110, 0.28);
        color: var(--accent);
      }
      .brush-shell {
        margin-top: 18px;
        padding: 16px 0 6px;
      }
      .brush-stage {
        position: relative;
        height: 118px;
        border-radius: 20px;
        background: linear-gradient(180deg, rgba(255,255,255,0.42), rgba(241,244,246,0.88));
        border: 1px solid rgba(26, 32, 43, 0.07);
        overflow: hidden;
      }
      .brush-chart {
        position: absolute;
        inset: 0;
      }
      .brush-selection {
        position: absolute;
        top: 10px;
        bottom: 10px;
        border-radius: 16px;
        background: rgba(15, 118, 110, 0.12);
        border: 2px solid rgba(15, 118, 110, 0.45);
        box-shadow: inset 0 0 0 999px rgba(255,255,255,0.1);
        cursor: grab;
      }
      .brush-selection.dragging {
        cursor: grabbing;
      }
      .brush-selection::before,
      .brush-selection::after {
        content: "";
        position: absolute;
        top: 0;
        bottom: 0;
        width: 12px;
        background: linear-gradient(180deg, rgba(255,255,255,0.9), rgba(230, 236, 239, 0.96));
      }
      .brush-selection::before {
        left: -12px;
        box-shadow: -999px 0 0 rgba(255,255,255,0.48);
      }
      .brush-selection::after {
        right: -12px;
        box-shadow: 999px 0 0 rgba(255,255,255,0.48);
      }
      .brush-handle {
        position: absolute;
        top: 50%;
        width: 18px;
        height: 42px;
        margin-top: -21px;
        border-radius: 999px;
        background: rgba(255,255,255,0.96);
        border: 1px solid rgba(26, 32, 43, 0.18);
        box-shadow: 0 10px 22px rgba(26, 32, 43, 0.14);
        cursor: ew-resize;
      }
      .brush-handle::before {
        content: "";
        position: absolute;
        top: 11px;
        bottom: 11px;
        left: 50%;
        width: 2px;
        margin-left: -1px;
        background: rgba(26, 32, 43, 0.28);
        box-shadow: -4px 0 0 rgba(26, 32, 43, 0.18), 4px 0 0 rgba(26, 32, 43, 0.18);
      }
      .brush-handle.start { left: -10px; }
      .brush-handle.end { right: -10px; }
      .brush-labels {
        display: flex;
        justify-content: space-between;
        gap: 16px;
        margin-top: 10px;
        color: var(--muted);
        font-size: 0.86rem;
      }
      .charts {
        grid-template-columns: repeat(2, minmax(0, 1fr));
        margin-bottom: 18px;
      }
      .chart-box {
        height: 210px;
        margin-top: 14px;
        border-radius: 18px;
        background: linear-gradient(180deg, rgba(255,255,255,0.4), rgba(242,245,247,0.85));
        border: 1px solid rgba(26, 32, 43, 0.07);
        display: grid;
        place-items: center;
        overflow: hidden;
        position: relative;
      }
      .disks table {
        width: 100%;
        border-collapse: collapse;
      }
      .disks th, .disks td {
        padding: 12px 0;
        text-align: left;
        border-bottom: 1px solid rgba(26, 32, 43, 0.08);
      }
      .disks th {
        color: var(--muted);
        font-size: 0.83rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
      }
      .empty {
        color: var(--muted);
      }
      @media (max-width: 960px) {
        .stats, .charts { grid-template-columns: 1fr; }
      }
      @media (max-width: 720px) {
        .shell { width: calc(100vw - 20px); }
        .topline { display: block; }
        .topline-actions {
          justify-content: space-between;
          margin-top: 12px;
        }
        .controls-head { display: block; }
        .controls-actions {
          justify-content: start;
          margin-top: 14px;
        }
        .preset-group {
          width: 100%;
          flex-wrap: wrap;
          border-radius: 18px;
        }
        .preset-button {
          flex: 1 0 50%;
        }
        .toggle-button { width: 100%; }
      }
    </style>
  </head>
  <body>
    <div class="shell">
      <div class="topline">
        <a class="back" href="/">← <span data-i18n="node.back">Back to dashboard</span></a>
        <div class="topline-actions">
          <label class="lang-picker">
            <span data-i18n="common.language">Language</span>
            <select id="language-select" class="lang-select" aria-label="Language"></select>
          </label>
          <div id="updated" class="label">Waiting for node data…</div>
        </div>
      </div>

      <section class="hero">
        <h1 id="title" data-i18n="node.loading">Loading node…</h1>
        <div class="meta" id="meta"></div>
      </section>

      <section class="stats" id="stats"></section>

      <section class="panel controls-panel">
        <div class="controls-head">
          <div>
            <div class="label" data-i18n="node.history_window">History Window</div>
            <div class="control-value" id="history-window-value">Last 24 hours</div>
            <div class="control-subtle" id="history-window-range">--</div>
          </div>
          <div class="controls-actions">
            <div class="preset-group" id="history-presets"></div>
            <button type="button" class="toggle-button" id="peak-clip-toggle">Clip Spikes: Off</button>
          </div>
        </div>
        <div class="brush-shell">
          <div class="brush-stage" id="history-brush-stage">
            <div class="brush-chart" id="history-brush-chart"></div>
            <div class="brush-selection" id="history-brush-selection">
              <div class="brush-handle start" data-handle="start"></div>
              <div class="brush-handle end" data-handle="end"></div>
            </div>
          </div>
          <div class="brush-labels">
            <span id="history-brush-start">--</span>
            <span id="history-brush-end">--</span>
          </div>
        </div>
      </section>

      <section class="charts">
        <article class="panel">
          <div class="label" data-i18n="node.cpu_usage">CPU Usage</div>
          <div class="chart-box" id="chart-cpu"></div>
        </article>
        <article class="panel">
          <div class="label" data-i18n="node.memory_usage">Memory Usage</div>
          <div class="chart-box" id="chart-memory"></div>
        </article>
        <article class="panel">
          <div class="label" data-i18n="node.download_upload">Download / Upload</div>
          <div class="chart-box" id="chart-network"></div>
        </article>
        <article class="panel">
          <div class="label" data-i18n="node.websocket_rtt">WebSocket RTT</div>
          <div class="chart-box" id="chart-latency"></div>
        </article>
      </section>

      <section class="panel disks">
        <div class="label" data-i18n="node.mounted_disks">Mounted Disks</div>
        <div id="disks" style="margin-top: 14px;"></div>
      </section>
    </div>

    <script>
      const NODE_ID = __NODE_ID_JSON__;
      const REFRESH_MS = __REFRESH_MS__;
      const I18N_ASSET_PATH = "__I18N_ASSET_PATH__";
      const LANGUAGE_STORAGE_KEY = "ximonitor.ui.language";
      const RETENTION_WINDOW_HOURS = 24 * 14;
      const OVERVIEW_HISTORY_MAX_POINTS = 1440;
      const DETAIL_HISTORY_MAX_POINTS = 720;
      const MIN_SELECTION_MS = 30 * 60 * 1000;
      const CUSTOM_PRESET_KEY = "custom";
      const PRESET_WINDOWS = [
        { key: "last_24h", hours: 24 },
        { key: "last_3d", hours: 72 },
        { key: "last_7d", hours: 168 },
        { key: "last_14d", hours: 336 }
      ];
      let I18N = { en: { "__label": "English" } };
      let currentLanguage = "en";
      let latestNode = null;
      let overviewHistory = [];
      let latestHistory = [];
      let refreshTimer = null;
      let activeBrushDrag = null;
      let detailRequestVersion = 0;
      const chartState = {
        peakClipEnabled: false,
        activePresetKey: "last_24h",
        selectedStartMs: null,
        selectedEndMs: null
      };

      function escapeHtml(value) {
        return String(value)
          .replaceAll("&", "&amp;")
          .replaceAll("<", "&lt;")
          .replaceAll(">", "&gt;")
          .replaceAll('"', "&quot;")
          .replaceAll("'", "&#39;");
      }

      function templateText(value, vars = {}) {
        return String(value).replace(/\{(\w+)\}/g, (_, key) => String(vars[key] ?? ""));
      }

      function supportedLanguages() {
        return Object.keys(I18N).filter((key) => key && typeof I18N[key] === "object");
      }

      function resolveLanguage(candidate) {
        const languages = supportedLanguages();
        if (candidate && languages.includes(candidate)) {
          return candidate;
        }
        const base = String(candidate || "").split("-")[0].toLowerCase();
        const matched = languages.find((language) => language.toLowerCase().startsWith(base));
        return matched || (languages.includes("en") ? "en" : languages[0] || "en");
      }

      function t(key, vars = {}) {
        const primary = I18N[currentLanguage] || {};
        const fallback = I18N.en || {};
        return templateText(primary[key] ?? fallback[key] ?? key, vars);
      }

      function languageLabel(language) {
        return (I18N[language] && I18N[language].__label) || language;
      }

      function storeLanguage(language) {
        try {
          window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
        } catch (_error) {
          // 在隐私模式或受限浏览器中 localStorage 可能不可用,这里静默忽略。
        }
      }

      function loadStoredLanguage() {
        try {
          return window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
        } catch (_error) {
          return null;
        }
      }

      async function loadI18n() {
        try {
          const response = await fetch(I18N_ASSET_PATH, {
            headers: { "accept": "application/json" },
          });
          if (!response.ok) {
            throw new Error(`${I18N_ASSET_PATH} -> ${response.status}`);
          }
          I18N = await response.json();
        } catch (error) {
          console.warn("failed to load ui translations", error);
        }
        currentLanguage = resolveLanguage(loadStoredLanguage() || navigator.language);
        storeLanguage(currentLanguage);
      }

      function bindLanguageSelector(onChange) {
        const select = document.getElementById("language-select");
        const renderOptions = () => {
          select.innerHTML = supportedLanguages().map((language) => `
            <option value="${escapeHtml(language)}">${escapeHtml(languageLabel(language))}</option>
          `).join("");
          select.value = currentLanguage;
        };

        renderOptions();
        select.addEventListener("change", (event) => {
          currentLanguage = resolveLanguage(event.target.value);
          storeLanguage(currentLanguage);
          renderOptions();
          onChange();
        });
      }

      function fmtBytes(bytes) {
        if (bytes == null) return t("common.not_available");
        const units = ["B", "KB", "MB", "GB", "TB", "PB"];
        let value = Number(bytes);
        let index = 0;
        while (value >= 1024 && index < units.length - 1) {
          value /= 1024;
          index += 1;
        }
        return `${value.toFixed(value >= 100 || index === 0 ? 0 : 1)} ${units[index]}`;
      }

      function fmtRate(bytes) {
        if (bytes == null) return t("common.not_available");
        return `${fmtBytes(bytes)}/s`;
      }

      function fmtPercent(value) {
        if (value == null || Number.isNaN(Number(value))) return t("common.not_available");
        return `${Number(value).toFixed(1)}%`;
      }

      function fmtLatency(value) {
        if (value == null) return t("common.not_available");
        return `${Math.round(value)} ms`;
      }

      function fmtUptime(seconds) {
        if (seconds == null || Number.isNaN(Number(seconds))) {
          return t("common.not_available");
        }
        const totalHours = Math.floor(Number(seconds) / 3600);
        const days = Math.floor(totalHours / 24);
        const hours = totalHours % 24;
        if (days > 0) {
          return t("node.uptime.days_hours", { days, hours });
        }
        return t("node.uptime.hours", { hours: totalHours });
      }

      function fmtDateTime(value) {
        return new Date(value).toLocaleString(currentLanguage);
      }

      function fetchJson(url) {
        return fetch(url, { headers: { "accept": "application/json" } }).then((response) => {
          if (!response.ok) throw new Error(`${url} -> ${response.status}`);
          return response.json();
        });
      }

      function normalizeHistory(history) {
        if (!Array.isArray(history)) {
          return [];
        }
        return history
          .map((point) => ({
            ...point,
            _ts: Date.parse(point.recorded_at)
          }))
          .filter((point) => Number.isFinite(point._ts))
          .sort((left, right) => left._ts - right._ts);
      }

      function historyBounds(history) {
        if (!Array.isArray(history) || history.length === 0) {
          return null;
        }
        return {
          startMs: history[0]._ts,
          endMs: history[history.length - 1]._ts
        };
      }

      function findPreset(key) {
        return PRESET_WINDOWS.find((preset) => preset.key === key) || null;
      }

      function selectionDurationMs() {
        if (chartState.selectedStartMs == null || chartState.selectedEndMs == null) {
          return 0;
        }
        return Math.max(chartState.selectedEndMs - chartState.selectedStartMs, 0);
      }

      function formatDurationLabel(durationMs) {
        const totalMinutes = Math.max(1, Math.round(durationMs / 60000));
        const days = Math.floor(totalMinutes / (24 * 60));
        const hours = Math.floor((totalMinutes % (24 * 60)) / 60);
        const minutes = totalMinutes % 60;

        if (days > 0) {
          return t("node.window.span_days_hours", { days, hours });
        }
        if (hours > 0) {
          return t("node.window.span_hours_minutes", { hours, minutes });
        }
        return t("node.window.span_minutes", { minutes: totalMinutes });
      }

      function formatWindowHeadline() {
        const preset = findPreset(chartState.activePresetKey);
        if (preset) {
          if (preset.hours < 24) {
            return t("node.window.last_hours", { hours: preset.hours });
          }
          return t("node.window.last_days", { days: preset.hours / 24 });
        }
        return t("node.window.custom", { span: formatDurationLabel(selectionDurationMs()) });
      }

      function clampSelection(startMs, endMs) {
        const bounds = historyBounds(overviewHistory);
        if (!bounds) {
          return null;
        }
        if (bounds.endMs <= bounds.startMs) {
          return {
            startMs: bounds.startMs,
            endMs: bounds.endMs
          };
        }

        const minSpan = Math.min(MIN_SELECTION_MS, bounds.endMs - bounds.startMs);
        let clampedStart = Math.max(bounds.startMs, Math.min(startMs, bounds.endMs));
        let clampedEnd = Math.max(bounds.startMs, Math.min(endMs, bounds.endMs));

        if (clampedEnd < clampedStart) {
          const swap = clampedStart;
          clampedStart = clampedEnd;
          clampedEnd = swap;
        }

        if (clampedEnd - clampedStart < minSpan) {
          clampedEnd = Math.min(bounds.endMs, clampedStart + minSpan);
          clampedStart = Math.max(bounds.startMs, clampedEnd - minSpan);
        }

        return {
          startMs: clampedStart,
          endMs: clampedEnd
        };
      }

      function renderPresetButtons() {
        const root = document.getElementById("history-presets");
        root.innerHTML = PRESET_WINDOWS.map((preset) => `
          <button
            type="button"
            class="preset-button ${chartState.activePresetKey === preset.key ? "active" : ""}"
            data-preset-key="${escapeHtml(preset.key)}"
          >${escapeHtml(t(`node.preset.${preset.key}`))}</button>
        `).join("");
      }

      function renderBrushOverview() {
        const root = document.getElementById("history-brush-chart");
        if (!Array.isArray(overviewHistory) || overviewHistory.length === 0) {
          root.innerHTML = "";
          return;
        }

        const width = 1000;
        const height = 118;
        const paddingX = 8;
        const paddingY = 12;
        const values = overviewHistory
          .map((point) => point.cpu_usage_percent)
          .filter((value) => value != null && Number.isFinite(Number(value)))
          .map((value) => Number(value));

        if (values.length === 0) {
          root.innerHTML = "";
          return;
        }

        const min = Math.min(...values);
        const max = Math.max(...values);
        const span = Math.max(max - min, 1);

        let started = false;
        const linePath = overviewHistory.map((point, index) => {
          const value = point.cpu_usage_percent;
          if (value == null) {
            return null;
          }
          const x = paddingX + ((width - paddingX * 2) * index) / Math.max(overviewHistory.length - 1, 1);
          const y = height - paddingY - (((Number(value) - min) / span) * (height - paddingY * 2));
          const command = started ? "L" : "M";
          started = true;
          return `${command}${x.toFixed(1)},${y.toFixed(1)}`;
        }).filter(Boolean).join(" ");

        const areaPath = `${linePath} L ${width - paddingX},${height - paddingY} L ${paddingX},${height - paddingY} Z`;

        root.innerHTML = `
          <svg viewBox="0 0 ${width} ${height}" width="100%" height="100%" preserveAspectRatio="none" aria-hidden="true">
            <defs>
              <linearGradient id="brushGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="rgba(15,118,110,0.24)" />
                <stop offset="100%" stop-color="rgba(15,118,110,0.04)" />
              </linearGradient>
            </defs>
            <path d="${areaPath}" fill="url(#brushGradient)" stroke="none" />
            <path d="${linePath}" fill="none" stroke="rgba(15,118,110,0.78)" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        `;
      }

      function syncBrushSelection() {
        const selection = document.getElementById("history-brush-selection");
        const bounds = historyBounds(overviewHistory);
        if (!bounds || chartState.selectedStartMs == null || chartState.selectedEndMs == null) {
          selection.style.display = "none";
          document.getElementById("history-brush-start").textContent = "--";
          document.getElementById("history-brush-end").textContent = "--";
          return;
        }

        selection.style.display = "block";
        const totalSpan = Math.max(bounds.endMs - bounds.startMs, 1);
        const leftPercent = ((chartState.selectedStartMs - bounds.startMs) / totalSpan) * 100;
        const rightPercent = ((bounds.endMs - chartState.selectedEndMs) / totalSpan) * 100;
        selection.style.left = `${Math.max(0, leftPercent)}%`;
        selection.style.right = `${Math.max(0, rightPercent)}%`;
        document.getElementById("history-brush-start").textContent = fmtDateTime(chartState.selectedStartMs);
        document.getElementById("history-brush-end").textContent = fmtDateTime(chartState.selectedEndMs);
      }

      function syncControls() {
        document.getElementById("history-window-value").textContent = formatWindowHeadline();
        document.getElementById("history-window-range").textContent =
          chartState.selectedStartMs != null && chartState.selectedEndMs != null
            ? t("node.window.range", {
                start: fmtDateTime(chartState.selectedStartMs),
                end: fmtDateTime(chartState.selectedEndMs)
              })
            : "--";
        const toggle = document.getElementById("peak-clip-toggle");
        toggle.textContent = chartState.peakClipEnabled ? t("node.clip.on") : t("node.clip.off");
        toggle.classList.toggle("active", chartState.peakClipEnabled);
        renderPresetButtons();
        syncBrushSelection();
      }

      function quantile(values, ratio) {
        if (!Array.isArray(values) || values.length === 0) return null;
        const sorted = [...values].sort((left, right) => left - right);
        const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * ratio) - 1));
        return sorted[index];
      }

      function chartBounds(values, clipSpikes) {
        const actualMin = Math.min(...values);
        const actualMax = Math.max(...values);
        let displayMax = actualMax;
        let clipped = false;

        if (clipSpikes && values.length >= 12) {
          const clippedMax = quantile(values, 0.98);
          if (clippedMax != null && clippedMax > actualMin && clippedMax < actualMax) {
            displayMax = clippedMax;
            clipped = true;
          }
        }

        return {
          actualMin,
          actualMax,
          displayMin: actualMin,
          displayMax,
          clipped,
        };
      }

      function renderSparkline(points, colors, formatter, options = {}) {
        if (!Array.isArray(points) || points.length === 0) {
          return `<div class="empty">${escapeHtml(t("node.waiting_history"))}</div>`;
        }

        const width = 640;
        const height = 210;
        const padding = 16;
        const allValues = points.flatMap((point) => point.values).filter((value) => value != null);
        if (allValues.length === 0) {
          return `<div class="empty">${escapeHtml(t("node.no_numeric_history"))}</div>`;
        }

        const bounds = chartBounds(allValues, options.clipSpikes);
        const span = Math.max(bounds.displayMax - bounds.displayMin, 1);
        const series = colors.map((color, seriesIndex) => {
          let started = false;
          const path = points.map((point, pointIndex) => {
            const value = point.values[seriesIndex];
            if (value == null) return null;
            const plottedValue = Math.min(Math.max(value, bounds.displayMin), bounds.displayMax);
            const x = padding + ((width - padding * 2) * pointIndex) / Math.max(points.length - 1, 1);
            const y = height - padding - (((plottedValue - bounds.displayMin) / span) * (height - padding * 2));
            const command = started ? "L" : "M";
            started = true;
            return `${command}${x.toFixed(1)},${y.toFixed(1)}`;
          }).filter(Boolean).join(" ");
          return `<path d="${path}" fill="none" stroke="${color}" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round" />`;
        }).join("");

        const footer = bounds.clipped
          ? t("node.chart.clipped_range", {
              start: formatter(bounds.displayMin),
              end: formatter(bounds.displayMax),
              peak: formatter(bounds.actualMax),
            })
          : t("node.chart.range", {
              start: formatter(bounds.displayMin),
              end: formatter(bounds.actualMax),
            });

        return `
          <svg viewBox="0 0 ${width} ${height}" width="100%" height="100%" preserveAspectRatio="none" aria-hidden="true">
            <rect x="0" y="0" width="${width}" height="${height}" fill="transparent" />
            ${series}
          </svg>
          <div style="position:absolute;left:18px;bottom:16px;font-size:0.82rem;color:#5d6875;">${escapeHtml(footer)}</div>
        `;
      }

      function filterHistoryBySelection(history) {
        if (!Array.isArray(history) || history.length === 0) {
          return [];
        }
        if (chartState.selectedStartMs == null || chartState.selectedEndMs == null) {
          return history;
        }

        const filtered = history.filter((point) => (
          point._ts >= chartState.selectedStartMs && point._ts <= chartState.selectedEndMs
        ));
        if (filtered.length > 0) {
          return filtered;
        }

        let before = null;
        let after = null;
        for (const point of history) {
          if (point._ts < chartState.selectedStartMs) {
            before = point;
            continue;
          }
          if (point._ts > chartState.selectedEndMs) {
            after = point;
            break;
          }
        }

        return [before, after].filter(Boolean);
      }

      function renderStats(node) {
        const snapshot = node.snapshot || {};
        const memory = snapshot.memory || {};
        const cards = [
          [t("node.stats.cpu"), fmtPercent(snapshot.cpu_usage_percent)],
          [t("node.stats.load"), snapshot.load ? `${snapshot.load.one.toFixed(2)} / ${snapshot.load.five.toFixed(2)} / ${snapshot.load.fifteen.toFixed(2)}` : t("common.not_available")],
          [t("node.stats.download_upload"), `${fmtRate(snapshot.network?.rx_bytes_per_sec)} / ${fmtRate(snapshot.network?.tx_bytes_per_sec)}`],
          [t("node.stats.latency"), fmtLatency(node.latency_ms)],
          [t("node.stats.memory"), `${fmtBytes(memory.used_bytes)} / ${fmtBytes(memory.total_bytes)}`],
          [t("node.stats.swap"), `${fmtBytes(memory.swap_used_bytes)} / ${fmtBytes(memory.swap_total_bytes)}`],
          [t("node.stats.uptime"), fmtUptime(snapshot.uptime_secs)],
          [t("node.stats.agent"), node.identity.agent_version || t("common.not_available")],
        ];
        document.getElementById("stats").innerHTML = cards.map(([label, value]) => `
          <article class="panel">
            <div class="label">${escapeHtml(label)}</div>
            <div class="value">${escapeHtml(value)}</div>
          </article>
        `).join("");
      }

      function renderDisks(node) {
        const disks = node.snapshot?.disks || [];
        const root = document.getElementById("disks");
        if (disks.length === 0) {
          root.innerHTML = `<div class="empty">${escapeHtml(t("node.no_disks"))}</div>`;
          return;
        }
        root.innerHTML = `
          <table>
            <thead>
              <tr>
                <th>${escapeHtml(t("node.disk.device"))}</th>
                <th>${escapeHtml(t("node.disk.mount"))}</th>
                <th>${escapeHtml(t("node.disk.filesystem"))}</th>
                <th>${escapeHtml(t("node.disk.usage"))}</th>
                <th>${escapeHtml(t("node.disk.capacity"))}</th>
              </tr>
            </thead>
            <tbody>
              ${disks.map((disk) => `
                <tr>
                  <td>${escapeHtml(disk.device)}</td>
                  <td>${escapeHtml(disk.mount_point)}</td>
                  <td>${escapeHtml(disk.fs_type)}</td>
                  <td>${fmtPercent(disk.used_percent)}</td>
                  <td>${fmtBytes(disk.used_bytes)} / ${fmtBytes(disk.total_bytes)}</td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        `;
      }

      function displayedHistory() {
        if (Array.isArray(latestHistory) && latestHistory.length > 0) {
          return latestHistory;
        }
        return filterHistoryBySelection(overviewHistory);
      }

      function renderHistory(history) {
        document.getElementById("chart-cpu").innerHTML = renderSparkline(
          history.map((point) => ({ values: [point.cpu_usage_percent] })),
          ["var(--chart-a)"],
          (value) => `${value.toFixed(1)}%`,
          { clipSpikes: chartState.peakClipEnabled }
        );
        document.getElementById("chart-memory").innerHTML = renderSparkline(
          history.map((point) => ({ values: [point.memory_used_percent] })),
          ["var(--chart-b)"],
          (value) => `${value.toFixed(1)}%`,
          { clipSpikes: chartState.peakClipEnabled }
        );
        document.getElementById("chart-network").innerHTML = renderSparkline(
          history.map((point) => ({ values: [point.rx_bytes_per_sec, point.tx_bytes_per_sec] })),
          ["var(--chart-c)", "var(--chart-a)"],
          (value) => fmtRate(value),
          { clipSpikes: chartState.peakClipEnabled }
        );
        document.getElementById("chart-latency").innerHTML = renderSparkline(
          history.map((point) => ({ values: [point.latency_ms] })),
          ["var(--chart-d)"],
          (value) => `${Math.round(value)} ms`,
          { clipSpikes: chartState.peakClipEnabled }
        );
      }

      function renderNodeHeader(node) {
        document.getElementById("title").textContent = node.identity.node_label || t("common.node_unavailable");
        document.getElementById("meta").innerHTML = `
          ${escapeHtml(node.identity.node_id)} · ${escapeHtml(node.identity.hostname || t("common.unknown_host"))} ·
          ${escapeHtml(node.identity.os || t("common.unknown_os"))} ·
          ${escapeHtml(node.online ? t("common.online") : t("common.offline"))}
        `;
      }

      function renderUpdatedAt(node) {
        document.getElementById("updated").textContent = node.last_seen
          ? t("common.last_seen", { time: fmtDateTime(node.last_seen) })
          : t("common.no_heartbeat_yet");
      }

      function rerenderNode() {
        document.documentElement.lang = currentLanguage;
        document.title = t("node.page_title");
        document.querySelectorAll("[data-i18n]").forEach((element) => {
          element.textContent = t(element.dataset.i18n);
        });
        renderBrushOverview();
        syncControls();
        if (latestNode) {
          renderNodeHeader(latestNode);
          renderUpdatedAt(latestNode);
          renderStats(latestNode);
          renderDisks(latestNode);
        } else {
          document.getElementById("updated").textContent = t("common.waiting_for_node_data");
          document.getElementById("title").textContent = t("node.loading");
        }
        renderHistory(displayedHistory());
      }

      function scheduleRefresh() {
        if (refreshTimer != null) {
          window.clearTimeout(refreshTimer);
        }
        refreshTimer = window.setTimeout(refresh, REFRESH_MS);
      }

      function xToHistoryMs(clientX) {
        const bounds = historyBounds(overviewHistory);
        if (!bounds) {
          return null;
        }
        const stage = document.getElementById("history-brush-stage");
        const rect = stage.getBoundingClientRect();
        if (rect.width <= 0) {
          return bounds.endMs;
        }
        const ratio = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
        return bounds.startMs + ((bounds.endMs - bounds.startMs) * ratio);
      }

      function updateSelection(startMs, endMs, activePresetKey) {
        const selection = clampSelection(startMs, endMs);
        if (!selection) {
          return false;
        }
        chartState.selectedStartMs = selection.startMs;
        chartState.selectedEndMs = selection.endMs;
        chartState.activePresetKey = activePresetKey;
        return true;
      }

      function applyPresetWindow(presetKey, shouldFetch = true) {
        const preset = findPreset(presetKey);
        const bounds = historyBounds(overviewHistory);
        if (!preset || !bounds) {
          return;
        }
        const endMs = bounds.endMs;
        const startMs = Math.max(bounds.startMs, endMs - (preset.hours * 3600 * 1000));
        if (!updateSelection(startMs, endMs, preset.key)) {
          return;
        }
        latestHistory = filterHistoryBySelection(overviewHistory);
        rerenderNode();
        if (shouldFetch) {
          void fetchDetailHistory();
        }
      }

      function ensureSelectionState() {
        const bounds = historyBounds(overviewHistory);
        if (!bounds) {
          chartState.selectedStartMs = null;
          chartState.selectedEndMs = null;
          latestHistory = [];
          return;
        }

        if (chartState.activePresetKey !== CUSTOM_PRESET_KEY) {
          applyPresetWindow(chartState.activePresetKey, false);
          return;
        }

        if (chartState.selectedStartMs == null || chartState.selectedEndMs == null) {
          applyPresetWindow("last_24h", false);
          return;
        }

        const selection = clampSelection(chartState.selectedStartMs, chartState.selectedEndMs);
        if (selection) {
          chartState.selectedStartMs = selection.startMs;
          chartState.selectedEndMs = selection.endMs;
          latestHistory = filterHistoryBySelection(overviewHistory);
        }
      }

      async function fetchDetailHistory() {
        const selection = clampSelection(chartState.selectedStartMs, chartState.selectedEndMs);
        if (!selection) {
          latestHistory = [];
          rerenderNode();
          return;
        }

        const requestVersion = ++detailRequestVersion;
        const params = new URLSearchParams({
          start: String(Math.floor(selection.startMs / 1000)),
          end: String(Math.ceil(selection.endMs / 1000)),
          max_points: String(DETAIL_HISTORY_MAX_POINTS)
        });

        try {
          const history = normalizeHistory(
            await fetchJson(`/api/nodes/${encodeURIComponent(NODE_ID)}/history?${params.toString()}`)
          );
          if (requestVersion !== detailRequestVersion) {
            return;
          }
          latestHistory = history.length > 0 ? history : filterHistoryBySelection(overviewHistory);
        } catch (error) {
          console.warn("failed to refresh selected history; falling back to overview samples", error);
          if (requestVersion !== detailRequestVersion) {
            return;
          }
          latestHistory = filterHistoryBySelection(overviewHistory);
        }

        renderHistory(displayedHistory());
      }

      function beginBrushDrag(mode, event) {
        const bounds = historyBounds(overviewHistory);
        if (!bounds || chartState.selectedStartMs == null || chartState.selectedEndMs == null) {
          return;
        }

        const pointerMs = xToHistoryMs(event.clientX);
        if (pointerMs == null) {
          return;
        }

        const selectionElement = document.getElementById("history-brush-selection");
        selectionElement.classList.toggle("dragging", mode === "move");
        activeBrushDrag = {
          mode,
          pointerOffsetMs: pointerMs - chartState.selectedStartMs,
          selectionSpanMs: chartState.selectedEndMs - chartState.selectedStartMs
        };

        const onPointerMove = (moveEvent) => {
          if (!activeBrushDrag) {
            return;
          }
          const nextPointerMs = xToHistoryMs(moveEvent.clientX);
          if (nextPointerMs == null) {
            return;
          }

          if (activeBrushDrag.mode === "start") {
            updateSelection(nextPointerMs, chartState.selectedEndMs, CUSTOM_PRESET_KEY);
          } else if (activeBrushDrag.mode === "end") {
            updateSelection(chartState.selectedStartMs, nextPointerMs, CUSTOM_PRESET_KEY);
          } else {
            const nextStart = nextPointerMs - activeBrushDrag.pointerOffsetMs;
            updateSelection(
              nextStart,
              nextStart + activeBrushDrag.selectionSpanMs,
              CUSTOM_PRESET_KEY
            );
          }

          latestHistory = filterHistoryBySelection(overviewHistory);
          syncControls();
          renderHistory(displayedHistory());
        };

        const onPointerUp = () => {
          selectionElement.classList.remove("dragging");
          window.removeEventListener("pointermove", onPointerMove);
          window.removeEventListener("pointerup", onPointerUp);
          activeBrushDrag = null;
          void fetchDetailHistory();
        };

        window.addEventListener("pointermove", onPointerMove);
        window.addEventListener("pointerup", onPointerUp);
      }

      function bindControls() {
        document.getElementById("history-presets").addEventListener("click", (event) => {
          const target = event.target.closest("[data-preset-key]");
          if (!target) {
            return;
          }
          applyPresetWindow(target.dataset.presetKey, true);
        });

        document.getElementById("peak-clip-toggle").addEventListener("click", () => {
          chartState.peakClipEnabled = !chartState.peakClipEnabled;
          syncControls();
          renderHistory(displayedHistory());
        });

        const brushStage = document.getElementById("history-brush-stage");
        const brushSelection = document.getElementById("history-brush-selection");
        brushStage.addEventListener("pointerdown", (event) => {
          const handle = event.target.closest("[data-handle]");
          if (handle) {
            event.preventDefault();
            beginBrushDrag(handle.dataset.handle, event);
            return;
          }

          if (brushSelection.contains(event.target)) {
            event.preventDefault();
            beginBrushDrag("move", event);
            return;
          }

          const pointerMs = xToHistoryMs(event.clientX);
          if (pointerMs == null) {
            return;
          }
          const currentSpan = Math.max(selectionDurationMs(), MIN_SELECTION_MS);
          updateSelection(
            pointerMs - (currentSpan / 2),
            pointerMs + (currentSpan / 2),
            CUSTOM_PRESET_KEY
          );
          latestHistory = filterHistoryBySelection(overviewHistory);
          syncControls();
          renderHistory(displayedHistory());
          void fetchDetailHistory();
        });

        syncControls();
      }

      async function refresh() {
        let node;
        try {
          node = await fetchJson(`/api/nodes/${encodeURIComponent(NODE_ID)}`);
        } catch (error) {
          document.getElementById("title").textContent = t("common.node_unavailable");
          document.getElementById("meta").textContent = error.message;
          scheduleRefresh();
          return;
        }

        latestNode = node;

        try {
          const overviewParams = new URLSearchParams({
            window_hours: String(RETENTION_WINDOW_HOURS),
            max_points: String(OVERVIEW_HISTORY_MAX_POINTS)
          });
          overviewHistory = normalizeHistory(
            await fetchJson(`/api/nodes/${encodeURIComponent(NODE_ID)}/history?${overviewParams.toString()}`)
          );
        } catch (error) {
          console.warn("failed to refresh overview history", error);
          overviewHistory = [];
          latestHistory = [];
        }

        ensureSelectionState();
        rerenderNode();
        await fetchDetailHistory();
        scheduleRefresh();
      }

      async function init() {
        await loadI18n();
        bindLanguageSelector(() => {
          rerenderNode();
        });
        bindControls();
        rerenderNode();
        refresh();
      }

      init();
    </script>
  </body>
</html>
"#;
