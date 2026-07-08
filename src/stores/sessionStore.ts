import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

/* ───── 类型定义（与 Rust 端对应） ───── */

export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  created_at: string;
}

export interface Session {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  messages: Message[];
}

export interface SessionSummary {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

/* ───── Store ───── */

interface SessionStore {
  sessions: SessionSummary[];
  currentId: string | null;
  currentSession: Session | null;
  loading: boolean;

  loadSessions: () => Promise<void>;
  createSession: () => Promise<string>;
  switchSession: (id: string) => Promise<void>;
  deleteSession: (id: string) => Promise<void>;
  renameSession: (id: string, name: string) => Promise<void>;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  sessions: [],
  currentId: null,
  currentSession: null,
  loading: false,

  /** 加载会话列表 */
  loadSessions: async () => {
    try {
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      set({ sessions });
    } catch (e) {
      console.error("加载会话列表失败:", e);
    }
  },

  /** 创建新会话，返回其 ID */
  createSession: async () => {
    try {
      const session = await invoke<Session>("create_session");
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      set({
        sessions,
        currentId: session.id,
        currentSession: session,
      });
      return session.id;
    } catch (e) {
      console.error("创建会话失败:", e);
      return "";
    }
  },

  /** 切换到指定会话 */
  switchSession: async (id: string) => {
    if (get().currentId === id) return;
    set({ currentId: id, currentSession: null, loading: true });
    try {
      const session = await invoke<Session | null>("get_session", { id });
      set({ currentSession: session, loading: false });
    } catch (e) {
      console.error("加载会话失败:", e);
      set({ loading: false });
    }
  },

  /** 删除会话 */
  deleteSession: async (id: string) => {
    try {
      await invoke("delete_session", { id });
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      const state = get();
      // 如果删除的是当前会话，清空 current
      const isCurrent = state.currentId === id;
      set({
        sessions,
        currentId: isCurrent ? null : state.currentId,
        currentSession: isCurrent ? null : state.currentSession,
      });
    } catch (e) {
      console.error("删除会话失败:", e);
    }
  },

  /** 重命名会话 */
  renameSession: async (id: string, name: string) => {
    try {
      await invoke("rename_session", { id, name });
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      set({ sessions });
      // 如果重命名的是当前会话，也更新 currentSession
      const state = get();
      if (state.currentSession?.id === id) {
        set({
          currentSession: { ...state.currentSession, name },
        });
      }
    } catch (e) {
      console.error("重命名会话失败:", e);
    }
  },
}));
