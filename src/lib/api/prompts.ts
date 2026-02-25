import { invoke } from "@tauri-apps/api/core";
import type { AppId } from "./types";

export interface PromptApps {
  claude: boolean;
  codex: boolean;
  gemini: boolean;
  opencode: boolean;
}

export interface Prompt {
  id: string;
  name: string;
  content: string;
  description?: string;
  apps: PromptApps;
  createdAt?: number;
  updatedAt?: number;
}

export const promptsApi = {
  async getAllPrompts(): Promise<Record<string, Prompt>> {
    return await invoke("get_prompts");
  },

  async upsertPrompt(prompt: Prompt): Promise<void> {
    return await invoke("upsert_prompt", { prompt });
  },

  async deletePrompt(id: string): Promise<void> {
    return await invoke("delete_prompt", { id });
  },

  async toggleApp(id: string, app: AppId, enabled: boolean): Promise<void> {
    return await invoke("toggle_prompt_app", { id, app, enabled });
  },

  async importFromFile(app: AppId): Promise<string> {
    return await invoke("import_prompt_from_file", { app });
  },

  async getCurrentFileContent(app: AppId): Promise<string | null> {
    return await invoke("get_current_prompt_file_content", { app });
  },
};
