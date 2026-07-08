import { useState } from "react";
import { useUiStore } from "../stores/uiStore";
import { useSessionStore } from "../stores/sessionStore";
import { FileTree } from "./FileTree";

/** 会话列表 — 左侧栏 */
export function SessionList() {
  const open = useUiStore((s) => s.leftPanelOpen);
  const toggle = useUiStore((s) => s.toggleLeftPanel);
  const {
    sessions,
    currentId,
    createSession,
    switchSession,
    deleteSession,
    renameSession,
  } = useSessionStore();
  const [leftTab, setLeftTab] = useState<"sessions" | "files">("sessions");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");

  const handleNew = async () => {
    await createSession();
  };

  const handleRename = async (id: string) => {
    if (editName.trim()) {
      await renameSession(id, editName.trim());
    }
    setEditingId(null);
  };

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm("确定删除这个会话？")) {
      await deleteSession(id);
    }
  };

  const startRename = (id: string, currentName: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(id);
    setEditName(currentName);
  };

  return (
    <aside
      className={`panel panel-left ${open ? "panel-open" : "panel-closed"}`}
    >
      {open ? (
        <>
          <div className="panel-header">
            <div className="panel-tabs">
              <button
                className={`panel-tab ${leftTab === "sessions" ? "panel-tab-active" : ""}`}
                onClick={() => setLeftTab("sessions")}
              >
                💬
              </button>
              <button
                className={`panel-tab ${leftTab === "files" ? "panel-tab-active" : ""}`}
                onClick={() => setLeftTab("files")}
              >
                📁
              </button>
            </div>
            <button
              className="panel-collapse-btn"
              onClick={toggle}
              title="折叠侧栏"
            >
              ◀
            </button>
          </div>

          {leftTab === "sessions" ? (
            <>
          <button className="btn-new-chat" onClick={handleNew}>
            ＋ 新对话
          </button>

          <div className="session-list">
            {sessions.map((s) => (
              <div
                key={s.id}
                className={`session-item ${currentId === s.id ? "session-active" : ""}`}
                onClick={() => switchSession(s.id)}
              >
                <span className="session-icon">💬</span>

                {editingId === s.id ? (
                  <input
                    className="session-rename-input"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    onBlur={() => handleRename(s.id)}
                    onKeyDown={(e) =>
                      e.key === "Enter" && handleRename(s.id)
                    }
                    autoFocus
                    onClick={(e) => e.stopPropagation()}
                  />
                ) : (
                  <>
                    <span className="session-name">{s.name}</span>
                    <div className="session-actions">
                      <button
                        className="session-btn"
                        onClick={(e) => startRename(s.id, s.name, e)}
                        title="重命名"
                      >
                        ✎
                      </button>
                      <button
                        className="session-btn"
                        onClick={(e) => handleDelete(s.id, e)}
                        title="删除"
                      >
                        ✕
                      </button>
                    </div>
                  </>
                )}
              </div>
            ))}
          </div>

          <div className="panel-footer-hint">
            {sessions.length} 个会话
          </div>
            </>
          ) : (
            <div className="panel-section">
              <div className="panel-section-header">📁 文件</div>
              <FileTree />
            </div>
          )}
        </>
      ) : (
        <button
          className="panel-expand-btn"
          onClick={toggle}
          title="展开侧栏"
        >
          ▶
        </button>
      )}
    </aside>
  );
}
