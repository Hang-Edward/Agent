import { useState, useEffect } from "react";
import { useSkillStore } from "../stores/skillStore";

/** Skill 管理器 — 左侧栏 📋 Tab */
export function SkillManager() {
  const { skills, load, remove, getById } = useSkillStore();
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [desc, setDesc] = useState("");
  const [prompt, setPrompt] = useState("");
  const [viewSkill, setViewSkill] = useState<string | null>(null);
  const [viewDetail, setViewDetail] = useState<{ name: string; description: string; system_prompt: string } | null>(null);

  useEffect(() => {
    load();
  }, []);

  const handleCreate = async () => {
    if (!name.trim()) return;
    const { create } = useSkillStore.getState();
    await create(name.trim(), desc.trim(), prompt.trim());
    setName("");
    setDesc("");
    setPrompt("");
    setShowForm(false);
  };

  const handleView = async (id: string) => {
    if (viewSkill === id) {
      setViewSkill(null);
      setViewDetail(null);
      return;
    }
    setViewSkill(id);
    const skill = await getById(id);
    if (skill) setViewDetail(skill);
  };

  return (
    <div className="panel-section">
      <div className="panel-section-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span>📋 Skills</span>
        <button className="btn-new-chat" style={{ margin: 0, padding: "4px 10px", fontSize: 12 }}
          onClick={() => setShowForm(!showForm)}>
          ＋ 新建
        </button>
      </div>

      {/* 新建表单 */}
      {showForm && (
        <div className="skill-form">
          <input className="skill-input" placeholder="Skill 名称" value={name}
            onChange={(e) => setName(e.target.value)} />
          <input className="skill-input" placeholder="简短描述" value={desc}
            onChange={(e) => setDesc(e.target.value)} />
          <textarea className="skill-textarea" placeholder="System Prompt 模板内容（支持 {{变量}}）" value={prompt}
            onChange={(e) => setPrompt(e.target.value)} rows={4} />
          <div style={{ display: "flex", gap: 6, marginTop: 6 }}>
            <button className="btn-sm" onClick={() => setShowForm(false)}>取消</button>
            <button className="btn-sm" style={{ background: "var(--accent)", color: "#1e1e1e" }} onClick={handleCreate}>保存</button>
          </div>
        </div>
      )}

      {/* Skill 列表 */}
      <div className="skill-list">
        {skills.length === 0 && !showForm && (
          <div className="hint" style={{ padding: "12px", textAlign: "center" }}>暂无 Skill，点击上方新建</div>
        )}
        {skills.map((s) => (
          <div key={s.id}>
            <div className="skill-item" onClick={() => handleView(s.id)}>
              <div className="skill-item-main">
                <span className="skill-name">{s.name}</span>
                <span className="skill-desc">{s.description}</span>
              </div>
              <button className="session-btn" onClick={(e) => { e.stopPropagation(); remove(s.id); }} title="删除">✕</button>
            </div>

            {/* 展开详情 */}
            {viewSkill === s.id && viewDetail && (
              <div className="skill-detail">
                <pre className="skill-detail-prompt">{viewDetail.system_prompt}</pre>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
