import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export interface SkillSummary {
  id: string;
  name: string;
  description: string;
  created_at: string;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  created_at: string;
}

interface SkillStore {
  skills: SkillSummary[];
  editing: Skill | null;
  dialogOpen: boolean;
  load: () => Promise<void>;
  create: (name: string, description: string, system_prompt: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
  getById: (id: string) => Promise<Skill | null>;
  setEditing: (skill: Skill | null) => void;
  setDialogOpen: (open: boolean) => void;
}

export const useSkillStore = create<SkillStore>((set) => ({
  skills: [],
  editing: null,
  dialogOpen: false,

  load: async () => {
    try {
      const skills = await invoke<SkillSummary[]>("list_skills");
      set({ skills });
    } catch (e) {
      console.error("加载 Skills 失败:", e);
    }
  },

  create: async (name, description, system_prompt) => {
    try {
      await invoke("create_skill", { name, description, systemPrompt: system_prompt });
      const skills = await invoke<SkillSummary[]>("list_skills");
      set({ skills, dialogOpen: false, editing: null });
    } catch (e) {
      console.error("创建 Skill 失败:", e);
    }
  },

  remove: async (id) => {
    try {
      await invoke("delete_skill", { id });
      const skills = await invoke<SkillSummary[]>("list_skills");
      set({ skills });
    } catch (e) {
      console.error("删除 Skill 失败:", e);
    }
  },

  getById: async (id) => {
    try {
      return await invoke<Skill | null>("get_skill", { id });
    } catch (e) {
      console.error("获取 Skill 失败:", e);
      return null;
    }
  },

  setEditing: (skill) => set({ editing: skill }),
  setDialogOpen: (open) => set({ dialogOpen: open, editing: open ? null : null }),
}));
