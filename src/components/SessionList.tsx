import { useUiStore } from "../stores/uiStore";

/** 会话列表 — 左侧栏 */
export function SessionList() {
  const open = useUiStore((s) => s.leftPanelOpen);
  const toggle = useUiStore((s) => s.toggleLeftPanel);

  return (
    <aside className={`panel panel-left ${open ? "panel-open" : "panel-closed"}`}>
      {open ? (
        <>
          <div className="panel-header">
            <span className="panel-title">会话</span>
            <button className="panel-collapse-btn" onClick={toggle} title="折叠侧栏">
              ◀
            </button>
          </div>

          <button className="btn-new-chat">＋ 新对话</button>

          <div className="session-list">
            {["对话 1", "对话 2", "对话 3"].map((name) => (
              <div key={name} className="session-item">
                <span className="session-icon">💬</span>
                <span className="session-name">{name}</span>
              </div>
            ))}
          </div>

          <div className="panel-footer-hint">已保存到本地</div>
        </>
      ) : (
        <button className="panel-expand-btn" onClick={toggle} title="展开侧栏">
          ▶
        </button>
      )}
    </aside>
  );
}
