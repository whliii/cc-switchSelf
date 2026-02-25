import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { promptsApi } from "@/lib/api/prompts";
import type { Prompt } from "@/lib/api/prompts";
import type { AppId } from "@/lib/api/types";

/**
 * 查询所有提示词（全局）
 */
export function useAllPrompts() {
  return useQuery({
    queryKey: ["prompts"],
    queryFn: () => promptsApi.getAllPrompts(),
  });
}

/**
 * 新增或更新提示词
 */
export function useUpsertPrompt() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (prompt: Prompt) => promptsApi.upsertPrompt(prompt),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["prompts"] });
    },
  });
}

/**
 * 删除提示词
 */
export function useDeletePrompt() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => promptsApi.deletePrompt(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["prompts"] });
    },
  });
}

/**
 * 切换提示词在指定 app 的启用状态（互斥由后端保证）
 */
export function useTogglePromptApp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      app,
      enabled,
    }: {
      id: string;
      app: AppId;
      enabled: boolean;
    }) => promptsApi.toggleApp(id, app, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["prompts"] });
    },
  });
}
