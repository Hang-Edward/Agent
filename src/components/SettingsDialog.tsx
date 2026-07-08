import { useSettingsStore, PERMISSION_LEVELS, MODELS, type Settings } from "../stores/settingsStore";
import { useState, useEffect } from "react";

/** API Key 输入组件：密码框 + 显示/隐藏切换 */
function ApiKeyInput({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  const [show, setShow] = useState(false);

  return (
    <div className="field">
      <label>API Key</label>
      <div className="input-row">
        <input
          type={show ? "text" : "password"}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="输入 DeepSeek API Key..."
        />
        <button className="btn-sm" onClick={() => setShow(!show)}>
          {show ? "隐藏" : "显示"}
        </button>
      </div>
    </div>
  );
}

export function SettingsDialog() {
  const { settings, dialogOpen, setDialogOpen, save } = useSettingsStore();
  const [form, setForm] = useState<Settings>(settings);

  // 打开对话框时同步当前设置到表单
  useEffect(() => {
    if (dialogOpen) setForm({ ...settings });
  }, [dialogOpen, settings]);

  if (!dialogOpen) return null;

  const handleSave = async () => {
    try {
      await save(form);
    } catch {
      // 错误已在 store 中打印
    }
  };

  return (
    <div className="overlay" onClick={() => setDialogOpen(false)}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>设置</h2>
          <button className="btn-close" onClick={() => setDialogOpen(false)}>
            ✕
          </button>
        </div>

        <div className="dialog-body">
          {/* API Key */}
          <ApiKeyInput
            value={form.api_key}
            onChange={(v) => setForm({ ...form, api_key: v })}
          />

          {/* 模型选择 */}
          <div className="field">
            <label>模型</label>
            <select
              value={form.model}
              onChange={(e) => setForm({ ...form, model: e.target.value })}
            >
              {MODELS.map((m) => (
                <option key={m.value} value={m.value}>
                  {m.label}
                </option>
              ))}
            </select>
          </div>

          {/* 权限级别 */}
          <div className="field">
            <label>权限级别</label>
            <div className="radio-group">
              {PERMISSION_LEVELS.map((p) => (
                <label key={p.value} className="radio-item">
                  <input
                    type="radio"
                    name="permission"
                    value={p.value}
                    checked={form.permission_level === p.value}
                    onChange={() =>
                      setForm({ ...form, permission_level: p.value })
                    }
                  />
                  <div>
                    <div className="radio-label">{p.label}</div>
                    <div className="radio-desc">{p.desc}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* 工作目录 */}
          <div className="field">
            <label>工作目录（可选）</label>
            <input
              type="text"
              value={form.working_dir}
              onChange={(e) => setForm({ ...form, working_dir: e.target.value })}
              placeholder="留空则使用当前项目目录"
            />
          </div>
        </div>

        <div className="dialog-footer">
          <button className="btn" onClick={() => setDialogOpen(false)}>
            取消
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
