import { useSettingsStore, MODELS, PERMISSION_LEVELS } from "../stores/settingsStore";

export function StatusBar() {
  const { settings, setDialogOpen } = useSettingsStore();

  const modelLabel = MODELS.find((m) => m.value === settings.model)?.label ?? settings.model;
  const permLabel = PERMISSION_LEVELS.find((p) => p.value === settings.permission_level)?.label ?? "";

  return (
    <div className="status-bar">
      <div className="status-left">
        <span className="status-item" title="当前模型">
          {modelLabel}
        </span>
        <span className="status-divider">|</span>
        <span className="status-item" title="权限级别">
          {permLabel}
        </span>
        {settings.api_key && (
          <>
            <span className="status-divider">|</span>
            <span className="status-item status-ok">API 已配置</span>
          </>
        )}
      </div>
      <div className="status-right">
        <button
          className="btn-gear"
          onClick={() => setDialogOpen(true)}
          title="设置"
        >
          ⚙
        </button>
      </div>
    </div>
  );
}
