import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export interface OpenFile {
  path: string;
  name: string;
  content: string;
  language?: string;
}

export type RightTab = "dashboard" | "editor" | "terminal";

interface FileStore {
  openFile: OpenFile | null;
  rightTab: RightTab;

  openFileByPath: (path: string, name: string) => Promise<void>;
  closeFile: () => void;
  setRightTab: (tab: RightTab) => void;
}

export const useFileStore = create<FileStore>((set) => ({
  openFile: null,
  rightTab: "dashboard",

  openFileByPath: async (path: string, name: string) => {
    try {
      const content = await invoke<string>("read_file_content", { path });
      set({ openFile: { path, name, content }, rightTab: "editor" });
    } catch (e) {
      console.error("打开文件失败:", e);
    }
  },

  closeFile: () => set({ openFile: null }),

  setRightTab: (tab) => set({ rightTab: tab }),
}));
