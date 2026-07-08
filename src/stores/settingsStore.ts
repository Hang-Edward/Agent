import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

/** 设置接口，与 Rust 后端 Settings 结构体对应 */
export interface Settings {
  api_key: string;
  model: string;
  permission_level: "default" | "review" | "full";
  working_dir: string;
}

/** 权限级别展示名称 */
export const PERMISSION_LEVELS = [
  { value: "default" as const, label: "默认模式", desc: "高风险操作弹窗确认" },
  { value: "review" as const, label: "自动审查", desc: "自动执行并审查变更" },
  { value: "full" as const, label: "完全访问", desc: "YOLO 模式，一键放行" },
];

/** 支持的模型列表 */
export const MODELS = [
  { value: "deepseek-v4-flash", label: "DeepSeek V4 Flash" },
  { value: "deepseek-v4-pro", label: "DeepSeek V4 Pro" },
];

interface SettingsStore {
  settings: Settings;
  dialogOpen: boolean;
  loaded: boolean;

  load: () => Promise<void>;
  save: (s: Settings) => Promise<void>;
  setDialogOpen: (open: boolean) => void;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: {
    api_key: "",
    model: "deepseek-v4-flash",
    permission_level: "default",
    working_dir: "",
  },
  dialogOpen: false,
  loaded: false,

  /** 从 Rust 后端加载设置 */
  load: async () => {
    try {
      const settings = await invoke<Settings>("get_settings");
      set({ settings, loaded: true });
    } catch (e) {
      console.error("加载设置失败:", e);
    }
  },

  /** 保存设置到 Rust 后端 */
  save: async (newSettings: Settings) => {
    try {
      await invoke("save_settings", { settings: newSettings });
      set({ settings: newSettings, dialogOpen: false });
    } catch (e) {
      console.error("保存设置失败:", e);
      throw e;
    }
  },

  setDialogOpen: (open: boolean) => set({ dialogOpen: open }),
}));
