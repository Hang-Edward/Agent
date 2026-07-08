import { useUiStore } from "../stores/uiStore";
import { useSessionStore } from "../stores/sessionStore";
import { useFileStore } from "../stores/fileStore";
import { MonacoEditor } from "./MonacoEditor";
import { TerminalPanel } from "./TerminalPanel";

/* ───────── 各仪表盘小组件 ───────── */

function TaskProgress() {
  return (
    <div className="widget">
      <div className="widget-title">📋 任务步骤</div>
      <div className="widget-body">
        <div className="hint" style={{ padding: "8px 0", textAlign: "center" }}>
          暂无活跃任务
        </div>
      </div>
    </div>
  );
}

function TokenUsage() {
  const stats = useSessionStore((s) => s.tokenStats);
  const total = stats.total_input + stats.total_output;

  return (
    <div className="widget">
      <div className="widget-title">🔢 Token 消耗</div>
      <div className="widget-body">
        <div className="metric-row">
          <span className="metric-label">输入</span>
          <span className="metric-value">{stats.total_input.toLocaleString()}</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">输出</span>
          <span className="metric-value">{stats.total_output.toLocaleString()}</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">总计</span>
          <span className="metric-value">{total.toLocaleString()}</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">预估费用</span>
          <span className="metric-value">${stats.total_cost.toFixed(4)}</span>
        </div>
      </div>
    </div>
  );
}

/** SVG 环形进度组件 */
function RingProgress({ pct, size = 80 }: { pct: number; size?: number }) {
  const stroke = 4;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (pct / 100) * circumference;
  const center = size / 2;

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <circle cx={center} cy={center} r={radius}
        fill="none" stroke="var(--border-light)" strokeWidth={stroke} />
      <circle cx={center} cy={center} r={radius}
        fill="none" stroke="url(#ctxGradient)" strokeWidth={stroke}
        strokeDasharray={circumference}
        strokeDashoffset={offset}
        strokeLinecap="round"
        transform={`rotate(-90 ${center} ${center})`}
        style={{ transition: "stroke-dashoffset 0.5s ease" }} />
      <defs>
        <linearGradient id="ctxGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="var(--accent)" />
          <stop offset="100%" stopColor="var(--purple)" />
        </linearGradient>
      </defs>
      <text x={center} y={center - 4}
        textAnchor="middle" dominantBaseline="middle"
        fill="var(--text-primary)" fontSize="16" fontWeight="bold"
        fontFamily="'Cascadia Code', monospace">
        {pct.toFixed(1)}%
      </text>
      <text x={center} y={center + 14}
        textAnchor="middle" dominantBaseline="middle"
        fill="var(--text-muted)" fontSize="10">
        used
      </text>
    </svg>
  );
}

/** 上下文使用量面板 — 环形展示 */
function ContextUsage() {
  const ctxTokens = useSessionStore((s) => s.contextTokens);
  const pct = Math.min(100, (ctxTokens / 1_000_000) * 100);

  return (
    <div className="widget">
      <div className="widget-title">📦 上下文 (1M)</div>
      <div className="widget-body" style={{ alignItems: "center" }}>
        <RingProgress pct={pct} />
        <div className="metric-row" style={{ width: "100%" }}>
          <span className="metric-label">已用</span>
          <span className="metric-value">{ctxTokens.toLocaleString()}</span>
        </div>
        <div className="metric-row" style={{ width: "100%" }}>
          <span className="metric-label">剩余</span>
          <span className="metric-value">{(1_000_000 - ctxTokens).toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}

function CacheHitRate() {
  return (
    <div className="widget">
      <div className="widget-title">⚡ 缓存命中率</div>
      <div className="widget-body">
        <div className="metric-row">
          <span className="metric-label">命中率</span>
          <span className="metric-value metric-highlight">—</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">本节省 (est.)</span>
          <span className="metric-value">$ —</span>
        </div>
        <div className="hint">DeepSeek API 暂未返回缓存统计</div>
      </div>
    </div>
  );
}

function GitStatus() {
  return (
    <div className="widget">
      <div className="widget-title">📂 Git 状态</div>
      <div className="widget-body">
        <div className="metric-row">
          <span className="metric-label">分支</span>
          <span className="metric-value">—</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">变更文件</span>
          <span className="metric-value">—</span>
        </div>
        <div className="hint">打开项目目录后自动检测</div>
      </div>
    </div>
  );
}

function DiffOverview() {
  return (
    <div className="widget">
      <div className="widget-title">📝 代码变更</div>
      <div className="widget-body">
        <div className="diff-bar">
          <div className="diff-added" style={{ flex: 0 }} />
          <div className="diff-removed" style={{ flex: 0 }} />
        </div>
        <div className="metric-row">
          <span className="metric-label metric-added">+ 新增</span>
          <span className="metric-value">0</span>
        </div>
        <div className="metric-row">
          <span className="metric-label metric-removed">− 删除</span>
          <span className="metric-value">0</span>
        </div>
        <div className="hint">文件变更后自动统计</div>
      </div>
    </div>
  );
}

/* ───────── 仪表盘容器 ───────── */

export function Dashboard() {
  const open = useUiStore((s) => s.rightPanelOpen);
  const toggle = useUiStore((s) => s.toggleRightPanel);
  const { rightTab, setRightTab, openFile, closeFile } = useFileStore();

  return (
    <aside className={`panel panel-right ${open ? "panel-open" : "panel-closed"}`}>
      {open ? (
        <>
          <div className="panel-header">
            <div className="panel-tabs">
              <button
                className={`panel-tab ${rightTab === "dashboard" ? "panel-tab-active" : ""}`}
                onClick={() => setRightTab("dashboard")}
              >
                📊
              </button>
              <button
                className={`panel-tab ${rightTab === "editor" ? "panel-tab-active" : ""}`}
                onClick={() => setRightTab("editor")}
              >
                ✏️
              </button>
              <button
                className={`panel-tab ${rightTab === "terminal" ? "panel-tab-active" : ""}`}
                onClick={() => setRightTab("terminal")}
              >
                💻
              </button>
            </div>
            <button className="panel-collapse-btn" onClick={toggle} title="折叠面板">
              ▶
            </button>
          </div>

          {rightTab === "dashboard" ? (
            <div className="dashboard-scroll">
              <TaskProgress />
              <TokenUsage />
              <ContextUsage />
              <CacheHitRate />
              <GitStatus />
              <DiffOverview />
            </div>
          ) : rightTab === "terminal" ? (
            <TerminalPanel />
          ) : (
            <div className="editor-container">
              {openFile ? (
                <>
                  <div className="editor-tab-header">
                    <span>{openFile.name}</span>
                    <button className="editor-tab-close" onClick={closeFile}>
                      ✕
                    </button>
                  </div>
                  <MonacoEditor
                    path={openFile.path}
                    content={openFile.content}
                  />
                </>
              ) : (
                <div className="welcome" style={{ padding: 20 }}>
                  <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
                    从文件树中选择一个文件预览
                  </p>
                </div>
              )}
            </div>
          )}
        </>
      ) : (
        <button className="panel-expand-btn" onClick={toggle} title="展开面板">
          ◀
        </button>
      )}
    </aside>
  );
}
