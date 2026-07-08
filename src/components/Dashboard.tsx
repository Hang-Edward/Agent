import { useUiStore } from "../stores/uiStore";

/* ───────── 各仪表盘小组件 ───────── */

/** 任务进度面板 */
function TaskProgress() {
  return (
    <div className="widget">
      <div className="widget-title">📋 任务进度</div>
      <div className="widget-body">
        <div className="progress-step">准备就绪</div>
        <div className="progress-bar-bg">
          <div className="progress-bar-fill" style={{ width: "0%" }} />
        </div>
        <div className="progress-label">等待指令...</div>
      </div>
    </div>
  );
}

/** Token 消耗面板 */
function TokenUsage() {
  return (
    <div className="widget">
      <div className="widget-title">🔢 Token 消耗</div>
      <div className="widget-body">
        <div className="metric-row">
          <span className="metric-label">本轮</span>
          <span className="metric-value">—</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">总计</span>
          <span className="metric-value">—</span>
        </div>
        <div className="metric-row">
          <span className="metric-label">预估费用</span>
          <span className="metric-value">$ —</span>
        </div>
      </div>
    </div>
  );
}

/** 缓存命中率面板 */
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
          <span className="metric-label">节省 (est.)</span>
          <span className="metric-value">$ —</span>
        </div>
        <div className="hint">对话开始后自动统计</div>
      </div>
    </div>
  );
}

/** Git 状态面板 */
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

/** Diff 概览面板 */
function DiffOverview() {
  return (
    <div className="widget">
      <div className="widget-title">📝 代码变更</div>
      <div className="widget-body">
        <div className="diff-bar">
          <div className="diff-added" style={{ flex: 0 }} title="新增行" />
          <div className="diff-removed" style={{ flex: 0 }} title="删除行" />
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

  return (
    <aside className={`panel panel-right ${open ? "panel-open" : "panel-closed"}`}>
      {open ? (
        <>
          <div className="panel-header">
            <span className="panel-title">仪表盘</span>
            <button className="panel-collapse-btn" onClick={toggle} title="折叠面板">
              ▶
            </button>
          </div>

          <div className="dashboard-scroll">
            <TaskProgress />
            <TokenUsage />
            <CacheHitRate />
            <GitStatus />
            <DiffOverview />
          </div>
        </>
      ) : (
        <button className="panel-expand-btn" onClick={toggle} title="展开面板">
          ◀
        </button>
      )}
    </aside>
  );
}
