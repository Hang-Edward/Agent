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

/** DeepSeek API 返回结果 */
export interface ChatResult {
  content: string;
  input_tokens: number;
  output_tokens: number;
}

/** 累计 Token 统计 */
export interface TokenStats {
  total_input: number;
  total_output: number;
  total_cost: number;
}

/* ───── Store ───── */

interface SessionStore {
  sessions: SessionSummary[];
  currentId: string | null;
  currentSession: Session | null;
  loading: boolean;
  sending: boolean;
  tokenStats: TokenStats;

  loadSessions: () => Promise<void>;
  createSession: () => Promise<string>;
  switchSession: (id: string) => Promise<void>;
  deleteSession: (id: string) => Promise<void>;
  renameSession: (id: string, name: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
}

/* DeepSeek V4 价格（每百万 token 的 USD 价格） */
const PRICES: Record<string, { input: number; output: number }> = {
  "deepseek-v4-flash": { input: 0.15, output: 0.6 },
  "deepseek-v4-pro": { input: 2.0, output: 8.0 },
};

/** 计算费用 */
function calcCost(model: string, input: number, output: number): number {
  const p = PRICES[model] ?? PRICES["deepseek-v4-flash"];
  return (input / 1_000_000) * p.input + (output / 1_000_000) * p.output;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  sessions: [],
  currentId: null,
  currentSession: null,
  loading: false,
  sending: false,
  tokenStats: { total_input: 0, total_output: 0, total_cost: 0 },

  loadSessions: async () => {
    try {
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      set({ sessions });
    } catch (e) {
      console.error("加载会话列表失败:", e);
    }
  },

  createSession: async () => {
    try {
      const session = await invoke<Session>("create_session");
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      set({ sessions, currentId: session.id, currentSession: session });
      return session.id;
    } catch (e) {
      console.error("创建会话失败:", e);
      return "";
    }
  },

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

  deleteSession: async (id: string) => {
    try {
      await invoke("delete_session", { id });
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      const state = get();
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

  renameSession: async (id: string, name: string) => {
    try {
      await invoke("rename_session", { id, name });
      const sessions = await invoke<SessionSummary[]>("list_sessions");
      set({ sessions });
      const state = get();
      if (state.currentSession?.id === id) {
        set({ currentSession: { ...state.currentSession, name } });
      }
    } catch (e) {
      console.error("重命名会话失败:", e);
    }
  },

  /** 发送消息到 DeepSeek API */
  sendMessage: async (content: string) => {
    const { currentId, currentSession } = get();
    if (!currentId || !currentSession) return;

    // 本地先插入用户消息，让 UI 即时响应
    const tempMsg: Message = {
      id: "temp-" + Date.now(),
      role: "user",
      content,
      created_at: new Date().toLocaleString("zh-CN"),
    };

    set({
      sending: true,
      currentSession: {
        ...currentSession,
        messages: [...currentSession.messages, tempMsg],
      },
    });

    try {
      const result = await invoke<ChatResult>("send_message", {
        sessionId: currentId,
        content,
      });

      // 重新从后端加载完整会话（获得含 AI 回复的最新状态）
      const updated = await invoke<Session | null>("get_session", { id: currentId });
      if (updated) {
        // 更新累计统计
        const state = get();
        const newStats: TokenStats = {
          total_input: state.tokenStats.total_input + result.input_tokens,
          total_output: state.tokenStats.total_output + result.output_tokens,
          total_cost:
            state.tokenStats.total_cost +
            calcCost("deepseek-v4-flash", result.input_tokens, result.output_tokens),
        };

        set({
          currentSession: updated,
          sending: false,
          tokenStats: newStats,
        });

        // 更新会话列表（名称可能已变）
        const sessions = await invoke<SessionSummary[]>("list_sessions");
        set({ sessions });
      }
    } catch (e) {
      console.error("发送消息失败:", e);
      // 即使失败也重新加载以恢复状态
      const updated = await invoke<Session | null>("get_session", { id: currentId });
      if (updated) set({ currentSession: updated });
      set({ sending: false });
    }
  },
}));
