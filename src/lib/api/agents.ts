import { invoke } from "@tauri-apps/api/core";

export interface AgentDefinition {
  id: string;
  name: string;
  content: string;
  description?: string;
  apps: {
    claude: boolean;
    codex: boolean;
    gemini: boolean;
    opencode: boolean;
  };
  createdAt?: number;
  updatedAt?: number;
}

export type AgentDefinitionsMap = Record<string, AgentDefinition>;

export const AGENT_APP_IDS = ["claude", "codex", "gemini", "opencode"] as const;
export type AgentAppId = (typeof AGENT_APP_IDS)[number];

export const agentsApi = {
  /**
   * 获取所有 Agent 定义
   */
  async getAll(): Promise<AgentDefinitionsMap> {
    return await invoke("get_agent_definitions");
  },

  /**
   * 新增或更新 Agent 定义
   */
  async upsert(agent: AgentDefinition): Promise<void> {
    return await invoke("upsert_agent_definition", { agent });
  },

  /**
   * 删除 Agent 定义
   */
  async delete(id: string): Promise<boolean> {
    return await invoke("delete_agent_definition", { id });
  },

  /**
   * 切换 Agent 在指定工具的启用状态
   */
  async toggleApp(
    agentId: string,
    app: AgentAppId,
    enabled: boolean,
  ): Promise<void> {
    return await invoke("toggle_agent_app", { agentId, app, enabled });
  },
};
