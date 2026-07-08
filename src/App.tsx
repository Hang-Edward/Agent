import { useEffect } from "react";
import { useSettingsStore } from "./stores/settingsStore";
import { SettingsDialog } from "./components/SettingsDialog";
import { StatusBar } from "./components/StatusBar";
import "./App.css";

function App() {
  const load = useSettingsStore((s) => s.load);

  // 启动时加载设置
  useEffect(() => {
    load();
  }, [load]);

  return (
    <div className="app">
      {/* 主内容区（后续填充） */}
      <main className="main-content">
        <div className="placeholder">
          <h1>Agent PC</h1>
          <p>桌面 AI 编码 Agent</p>
        </div>
      </main>

      {/* 底部状态栏 */}
      <StatusBar />

      {/* 设置对话框 */}
      <SettingsDialog />
    </div>
  );
}

export default App;
