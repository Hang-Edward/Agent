import { useUiStore } from "../stores/uiStore";
import { useSessionStore } from "../stores/sessionStore";

/* ───────── 各仪表盘小组件 ───────── */

/** 任务步骤 */
interface TaskStep {
  id: string;
  label: string;
  status: "pending" | "running" | "done" | "failed";
}

/** 任务进度面板 — 多步骤清单模式 */
function TaskProgress() {
  // 示例数据，后续由 Agent 循环更新
  const steps: TaskStep[] = [
    { id: "1", label: "分析需求", status: "done" },
    { id: "2", label: "读取相关文件", status: "done" },
    { id: "3", label: "生成修改方案", status: "running" },
    { id: "4", label: "执行代码修改", status: "pending" },
    { id: "5", label: "验证修改结果", status: "pending" },
  ];

  return (
    <div className="widget">
      <div className="widget-title">📋 任务步骤</div>
      <div className="widget-body">
        <div className="step-list">
          {steps.map((step) => (
            <div
              key={step.id}
              className={`step-item step-${step.status}`}
            >
              <span className="step-icon">
                {step.status === "done" && "✓"}
                {step.status === "running" && "◌"}
                {step.status === "pending" && "○"}
                {step.status === "failed" && "✕"}
              </span>
              <span className={`step-label ${step.status === "done" ? "step-done" : ""}`}>
                {step.label}
              </span>
              {step.status === "running" && <span className="step-spinner" />}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

/** Token 消耗面板 */
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

/** 缓存命中率面板（DeepSeek API 暂未暴露缓存数据，显示占位） */
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
