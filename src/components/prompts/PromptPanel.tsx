import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { FileText, Edit3, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  useAllPrompts,
  useDeletePrompt,
  useTogglePromptApp,
} from "@/hooks/usePrompts";
import type { Prompt } from "@/lib/api/prompts";
import type { AppId } from "@/lib/api/types";
import { ConfirmDialog } from "../ConfirmDialog";
import { AppCountBar } from "@/components/common/AppCountBar";
import { AppToggleGroup } from "@/components/common/AppToggleGroup";
import { ListItemRow } from "@/components/common/ListItemRow";
import { MCP_SKILLS_APP_IDS } from "@/config/appConfig";
import PromptFormPanel from "./PromptFormPanel";

interface PromptPanelProps {
  onOpenChange: (open: boolean) => void;
}

export interface PromptPanelHandle {
  openAdd: () => void;
}

// Helper: convert prompt apps (4-key) to full Record<AppId, boolean>
function toAppRecord(apps: Prompt["apps"]): Record<AppId, boolean> {
  return { ...apps, openclaw: false };
}

const PromptPanel = React.forwardRef<PromptPanelHandle, PromptPanelProps>(
  ({ onOpenChange: _onOpenChange }, ref) => {
    const { t } = useTranslation();
    const [isFormOpen, setIsFormOpen] = useState(false);
    const [editingId, setEditingId] = useState<string | null>(null);
    const [confirmDialog, setConfirmDialog] = useState<{
      isOpen: boolean;
      title: string;
      message: string;
      onConfirm: () => void;
    } | null>(null);

    const { data: promptsMap, isLoading } = useAllPrompts();
    const toggleAppMutation = useTogglePromptApp();
    const deletePromptMutation = useDeletePrompt();

    const promptEntries = useMemo((): Array<[string, Prompt]> => {
      if (!promptsMap) return [];
      return Object.entries(promptsMap);
    }, [promptsMap]);

    const enabledCounts = useMemo(() => {
      const counts = {
        claude: 0,
        codex: 0,
        gemini: 0,
        opencode: 0,
        openclaw: 0,
      };
      promptEntries.forEach(([_, prompt]) => {
        for (const app of MCP_SKILLS_APP_IDS) {
          if (prompt.apps[app as keyof typeof prompt.apps]) counts[app]++;
        }
      });
      return counts;
    }, [promptEntries]);

    const handleToggleApp = async (
      promptId: string,
      app: AppId,
      enabled: boolean,
    ) => {
      if (app === "openclaw") return;
      try {
        await toggleAppMutation.mutateAsync({ id: promptId, app, enabled });
      } catch (error) {
        toast.error(t("common.error"), { description: String(error) });
      }
    };

    const handleEdit = (id: string) => {
      setEditingId(id);
      setIsFormOpen(true);
    };

    const handleAdd = () => {
      setEditingId(null);
      setIsFormOpen(true);
    };

    React.useImperativeHandle(ref, () => ({
      openAdd: handleAdd,
    }));

    const handleDelete = (id: string) => {
      const prompt = promptsMap?.[id];
      setConfirmDialog({
        isOpen: true,
        title: t("prompts.confirm.deleteTitle"),
        message: t("prompts.confirm.deleteMessage", { name: prompt?.name }),
        onConfirm: async () => {
          try {
            await deletePromptMutation.mutateAsync(id);
            setConfirmDialog(null);
            toast.success(t("common.success"), { closeButton: true });
          } catch (error) {
            toast.error(t("common.error"), { description: String(error) });
          }
        },
      });
    };

    const handleCloseForm = () => {
      setIsFormOpen(false);
      setEditingId(null);
    };

    return (
      <div className="px-6 flex flex-col h-[calc(100vh-8rem)] overflow-hidden">
        <AppCountBar
          totalLabel={t("prompts.count", { count: promptEntries.length })}
          counts={enabledCounts}
          appIds={MCP_SKILLS_APP_IDS}
        />

        <div className="flex-1 overflow-y-auto overflow-x-hidden pb-24">
          {isLoading ? (
            <div className="text-center py-12 text-muted-foreground">
              {t("common.loading")}
            </div>
          ) : promptEntries.length === 0 ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 mx-auto mb-4 bg-muted rounded-full flex items-center justify-center">
                <FileText size={24} className="text-muted-foreground" />
              </div>
              <h3 className="text-lg font-medium text-foreground mb-2">
                {t("prompts.empty")}
              </h3>
              <p className="text-muted-foreground text-sm">
                {t("prompts.emptyDescription")}
              </p>
            </div>
          ) : (
            <TooltipProvider delayDuration={300}>
              <div className="rounded-xl border border-border-default overflow-hidden">
                {promptEntries.map(([id, prompt], index) => (
                  <PromptListItem
                    key={id}
                    id={id}
                    prompt={prompt}
                    onToggleApp={handleToggleApp}
                    onEdit={handleEdit}
                    onDelete={handleDelete}
                    isLast={index === promptEntries.length - 1}
                  />
                ))}
              </div>
            </TooltipProvider>
          )}
        </div>

        {isFormOpen && (
          <PromptFormPanel
            editingId={editingId ?? undefined}
            initialData={editingId && promptsMap ? promptsMap[editingId] : undefined}
            onClose={handleCloseForm}
          />
        )}

        {confirmDialog && (
          <ConfirmDialog
            isOpen={confirmDialog.isOpen}
            title={confirmDialog.title}
            message={confirmDialog.message}
            onConfirm={confirmDialog.onConfirm}
            onCancel={() => setConfirmDialog(null)}
          />
        )}
      </div>
    );
  },
);

PromptPanel.displayName = "PromptPanel";

export default PromptPanel;

// ============================================================================
// List Item
// ============================================================================

interface PromptListItemProps {
  id: string;
  prompt: Prompt;
  onToggleApp: (promptId: string, app: AppId, enabled: boolean) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  isLast?: boolean;
}

const PromptListItem: React.FC<PromptListItemProps> = ({
  id,
  prompt,
  onToggleApp,
  onEdit,
  onDelete,
  isLast,
}) => {
  const { t } = useTranslation();
  const name = prompt.name || id;
  const description = prompt.description || "";

  return (
    <ListItemRow isLast={isLast}>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="font-medium text-sm text-foreground truncate">
            {name}
          </span>
        </div>
        {description && (
          <p
            className="text-xs text-muted-foreground truncate"
            title={description}
          >
            {description}
          </p>
        )}
      </div>

      <AppToggleGroup
        apps={toAppRecord(prompt.apps)}
        onToggle={(app, enabled) => onToggleApp(id, app, enabled)}
        appIds={MCP_SKILLS_APP_IDS}
      />

      <div className="flex items-center gap-0.5 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onEdit(id)}
          title={t("common.edit")}
        >
          <Edit3 size={14} />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-7 w-7 hover:text-red-500 hover:bg-red-100 dark:hover:text-red-400 dark:hover:bg-red-500/10"
          onClick={() => onDelete(id)}
          title={t("common.delete")}
        >
          <Trash2 size={14} />
        </Button>
      </div>
    </ListItemRow>
  );
};
