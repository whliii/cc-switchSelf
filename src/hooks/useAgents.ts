import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { agentsApi } from "@/lib/api/agents";
import type { AgentDefinition, AgentAppId } from "@/lib/api/agents";

/**
 * 查询所有 Agent 定义
 */
export function useAllAgents() {
  return useQuery({
    queryKey: ["agents", "all"],
    queryFn: () => agentsApi.getAll(),
  });
}

/**
 * 新增或更新 Agent 定义
 */
export function useUpsertAgent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (agent: AgentDefinition) => agentsApi.upsert(agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agents", "all"] });
    },
  });
}

/**
 * 删除 Agent 定义
 */
export function useDeleteAgent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => agentsApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agents", "all"] });
    },
  });
}

/**
 * 切换 Agent 在指定工具的启用状态
 */
export function useToggleAgentApp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      agentId,
      app,
      enabled,
    }: {
      agentId: string;
      app: AgentAppId;
      enabled: boolean;
    }) => agentsApi.toggleApp(agentId, app, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agents", "all"] });
    },
  });
}
