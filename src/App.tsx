import { useEffect } from "react";
import { useSettingsStore } from "./stores/settingsStore";
import { useSessionStore } from "./stores/sessionStore";
import { SessionList } from "./components/SessionList";
import { ChatArea } from "./components/ChatArea";
import { Dashboard } from "./components/Dashboard";
import { SettingsDialog } from "./components/SettingsDialog";
import { StatusBar } from "./components/StatusBar";
import "./App.css";

function App() {
  const loadSettings = useSettingsStore((s) => s.load);
  const loadSessions = useSessionStore((s) => s.loadSessions);

  useEffect(() => {
    loadSettings();
    loadSessions();
  }, [loadSettings, loadSessions]);

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
