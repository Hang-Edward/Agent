import { useEffect } from "react";
import { useSettingsStore } from "./stores/settingsStore";
import { SessionList } from "./components/SessionList";
import { ChatArea } from "./components/ChatArea";
import { Dashboard } from "./components/Dashboard";
import { SettingsDialog } from "./components/SettingsDialog";
import { StatusBar } from "./components/StatusBar";
import "./App.css";

function App() {
  const load = useSettingsStore((s) => s.load);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <div className="app">
      <div className="layout">
        <SessionList />
        <ChatArea />
        <Dashboard />
      </div>
      <StatusBar />
      <SettingsDialog />
    </div>
  );
}

export default App;
