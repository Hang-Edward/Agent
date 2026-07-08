import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

/* ───── 类型定义 ───── */

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

export interface ChatResult {
  content: string;
  input_tokens: number;
  output_tokens: number;
}

export interface TokenStats {
  total_input: number;
  total_output: number;
  total_cost: number;
}

/* ───── 流式事件载荷（与 Rust 端对应） ───── */

interface TokenPayload {
  token: string;
}
interface ReasoningPayload {
  reasoning: string;
}
interface ToolResultPayload {
  id: string;
  name: string;
  result: string;
}

/** 前端追踪的工具调用（用于 UI 展示） */
export interface ToolCallDisplay {
  id: string;
  name: string;
  args: string;
  result?: string;
  status: "running" | "success" | "error";
}

/* ───── 价格计算 ───── */

const PRICES: Record<string, { input: number; output: number }> = {
  "deepseek-v4-flash": { input: 0.15, output: 0.6 },
  "deepseek-v4-pro": { input: 2.0, output: 8.0 },
};

function calcCost(model: string, input: number, output: number): number {
  const p = PRICES[model] ?? PRICES["deepseek-v4-flash"];
  return (input / 1_000_000) * p.input + (output / 1_000_000) * p.output;
}

/* ───── Store ───── */

interface SessionStore {
  sessions: SessionSummary[];
  currentId: string | null;
  currentSession: Session | null;
  loading: boolean;
  sending: boolean;
  /** 实时流式内容（尚未保存到会话的 AI 回复文本） */
  streamContent: string;
  /** 实时思考过程 */
  streamReasoning: string;
  /** 本轮工具调用（用于 UI 展示） */
  toolCalls: ToolCallDisplay[];
  tokenStats: TokenStats;

  loadSessions: () => Promise<void>;
  createSession: () => Promise<string>;
  switchSession: (id: string) => Promise<void>;
  deleteSession: (id: string) => Promise<void>;
  renameSession: (id: string, name: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  sessions: [],
  currentId: null,
  currentSession: null,
  loading: false,
  sending: false,
  streamContent: "",
  streamReasoning: "",
  toolCalls: [],
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
    set({
      currentId: id,
      currentSession: null,
      loading: true,
      streamContent: "",
      streamReasoning: "",
      toolCalls: [],
    });
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

  /** 发送消息 → 启动 Agent Turn Loop（流式） */
  sendMessage: async (content: string) => {
    const { currentId, currentSession } = get();
    if (!currentId || !currentSession) return;

    // 1. 本地插入用户消息
    const tempUserMsg: Message = {
      id: "user-" + Date.now(),
      role: "user",
      content,
      created_at: new Date().toLocaleString("zh-CN"),
    };

    set({
      sending: true,
      streamContent: "",
      streamReasoning: "",
      toolCalls: [],
      currentSession: {
        ...currentSession,
        messages: [...currentSession.messages, tempUserMsg],
      },
    });

    // 2. 监听 Tauri 流式事件
    const unlisteners: UnlistenFn[] = [];
    let aiContent = "";

    try {
      // token 事件
      const un1 = await listen<TokenPayload>("agent:token", (ev) => {
        aiContent += ev.payload.token;
        set({ streamContent: aiContent });
      });
      unlisteners.push(un1);

      // reasoning 事件
      const un2 = await listen<ReasoningPayload>("agent:reasoning", (ev) => {
        set((s) => ({ streamReasoning: s.streamReasoning + ev.payload.reasoning }));
      });
      unlisteners.push(un2);

      // tool_result 事件
      const un3 = await listen<ToolResultPayload>("agent:tool_result", (ev) => {
        const tc: ToolCallDisplay = {
          id: ev.payload.id,
          name: ev.payload.name,
          args: "",
          result: ev.payload.result,
          status: "success",
        };
        set((s) => ({ toolCalls: [...s.toolCalls, tc] }));
      });
      unlisteners.push(un3);

      // 3. 启动 Agent Turn（异步，Rust 端通过事件推流）
      const result = await invoke<ChatResult>("start_agent_turn", {
        sessionId: currentId,
        content,
      });

      // 4. Turn 完成 → 重新加载最新会话
      const updated = await invoke<Session | null>("get_session", {
        id: currentId,
      });
      if (updated) {
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
          streamContent: "",
          streamReasoning: "",
          tokenStats: newStats,
        });

        const sessions = await invoke<SessionSummary[]>("list_sessions");
        set({ sessions });
      }
    } catch (e) {
      console.error("Agent Turn 失败:", e);
      set({ sending: false, streamContent: "", streamReasoning: "" });
      const updated = await invoke<Session | null>("get_session", { id: currentId });
      if (updated) set({ currentSession: updated });
    } finally {
      unlisteners.forEach((fn) => fn());
    }
  },
}));
