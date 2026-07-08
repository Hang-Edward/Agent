import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { confirm } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "../stores/uiStore";
import { useSessionStore } from "../stores/sessionStore";
import { SkillManager } from "./SkillManager";

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
  const [leftTab, setLeftTab] = useState<"sessions" | "skills">("sessions");
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
    const ok = await confirm("确定删除这个会话？", { title: "删除会话", kind: "warning" });
    if (ok) {
      await deleteSession(id);
    }
  };

  const handleOpenFolder = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("open_in_explorer", { path: "." });
    } catch (err) {
      console.error("打开文件夹失败:", err);
    }
  };

  const startRename = (id: string, currentName: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(id);
    setEditName(currentName);
  };

  return (
    <aside className={`panel panel-left ${open ? "panel-open" : "panel-closed"}`}>
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
                className={`panel-tab ${leftTab === "skills" ? "panel-tab-active" : ""}`}
                onClick={() => setLeftTab("skills")}
              >
                📋
              </button>
            </div>
            <button className="panel-collapse-btn" onClick={toggle} title="折叠侧栏">◀</button>
          </div>

          {leftTab === "sessions" ? (
            <>
              {/* 项目区 */}
              <div className="panel-section-header" style={{ padding: "10px 12px 4px" }}>
                📂 项目
              </div>
              <div className="project-card" onClick={handleOpenFolder} title="打开项目文件夹">
                <span className="project-icon">📁</span>
                <div className="project-info">
                  <span className="project-name">Agent PC</span>
                  <span className="project-path">点击打开文件夹</span>
                </div>
              </div>

              {/* 对话区 */}
              <div className="panel-section-header" style={{ padding: "12px 12px 4px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span>💬 对话</span>
                <button className="btn-new-chat" onClick={handleNew} style={{ margin: 0, padding: "3px 8px", fontSize: 11 }}>＋ 新建</button>
              </div>

              <div className="session-list">
                {sessions.length === 0 && (
                  <div className="hint" style={{ padding: "12px", textAlign: "center" }}>暂无对话</div>
                )}
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
                        onKeyDown={(e) => e.key === "Enter" && handleRename(s.id)}
                        autoFocus
                        onClick={(e) => e.stopPropagation()}
                      />
                    ) : (
                      <>
                        <span className="session-name">{s.name}</span>
                        <div className="session-actions">
                          <button className="session-btn" onClick={(e) => handleOpenFolder(e)} title="打开文件夹">
                            📂
                          </button>
                          <button className="session-btn" onClick={(e) => startRename(s.id, s.name, e)} title="重命名">
                            ✎
                          </button>
                          <button className="session-btn" onClick={(e) => handleDelete(s.id, e)} title="删除">
                            ✕
                          </button>
                        </div>
                      </>
                    )}
                  </div>
                ))}
              </div>

              <div className="panel-footer-hint">{sessions.length} 个对话</div>
            </>
          ) : (
            <SkillManager />
          )}
        </>
      ) : (
        <button className="panel-expand-btn" onClick={toggle} title="展开侧栏">▶</button>
      )}
    </aside>
  );
}
