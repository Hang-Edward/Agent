import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export interface OpenFile {
  path: string;
  name: string;
  content: string;
  language?: string;
}

interface FileStore {
  openFile: OpenFile | null;
  rightTab: "dashboard" | "editor";

  openFileByPath: (path: string, name: string) => Promise<void>;
  closeFile: () => void;
  setRightTab: (tab: "dashboard" | "editor") => void;
}

export const useFileStore = create<FileStore>((set) => ({
  openFile: null,
  rightTab: "dashboard",

  openFileByPath: async (path: string, name: string) => {
    try {
      // 通过沙箱读文件（复用 sandbox）
      const content = await invoke<string>("read_file_content", { path });
      set({ openFile: { path, name, content }, rightTab: "editor" });
    } catch (e) {
      console.error("打开文件失败:", e);
    }
  },

  closeFile: () => set({ openFile: null }),

  setRightTab: (tab) => set({ rightTab: tab }),
}));
