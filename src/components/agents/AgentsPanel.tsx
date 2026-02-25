import React, { useMemo, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Bot, Edit3, Trash2, Save } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  useAllAgents,
  useUpsertAgent,
  useDeleteAgent,
  useToggleAgentApp,
} from "@/hooks/useAgents";
import type { AgentDefinition, AgentAppId } from "@/lib/api/agents";
import type { AppId } from "@/lib/api/types";
import { ConfirmDialog } from "../ConfirmDialog";
import { AppCountBar } from "@/components/common/AppCountBar";
import { AppToggleGroup } from "@/components/common/AppToggleGroup";
import { ListItemRow } from "@/components/common/ListItemRow";
import { FullScreenPanel } from "@/components/common/FullScreenPanel";
import MarkdownEditor from "@/components/MarkdownEditor";
import { MCP_SKILLS_APP_IDS } from "@/config/appConfig";

interface AgentsPanelProps {
  onOpenChange: (open: boolean) => void;
}

export interface AgentsPanelHandle {
  openAdd: () => void;
}

// Helper: convert agent apps (4-key) to full Record<AppId, boolean>
function toAppRecord(
  apps: AgentDefinition["apps"],
): Record<AppId, boolean> {
  return { ...apps, openclaw: false };
}

// Helper: generate slug from name
function toSlug(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-");
}

const AgentsPanel = React.forwardRef<AgentsPanelHandle, AgentsPanelProps>(
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

    const { data: agentsMap, isLoading } = useAllAgents();
    const toggleAppMutation = useToggleAgentApp();
    const deleteAgentMutation = useDeleteAgent();

    const agentEntries = useMemo((): Array<[string, AgentDefinition]> => {
      if (!agentsMap) return [];
      return Object.entries(agentsMap);
    }, [agentsMap]);

    const enabledCounts = useMemo(() => {
      const counts = {
        claude: 0,
        codex: 0,
        gemini: 0,
        opencode: 0,
        openclaw: 0,
      };
      agentEntries.forEach(([_, agent]) => {
        for (const app of MCP_SKILLS_APP_IDS) {
          if (agent.apps[app as AgentAppId]) counts[app]++;
        }
      });
      return counts;
    }, [agentEntries]);

    const handleToggleApp = async (
      agentId: string,
      app: AppId,
      enabled: boolean,
    ) => {
      if (app === "openclaw") return;
      try {
        await toggleAppMutation.mutateAsync({
          agentId,
          app: app as AgentAppId,
          enabled,
        });
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
      setConfirmDialog({
        isOpen: true,
        title: t("agents.deleteTitle"),
        message: t("agents.deleteConfirm", { id }),
        onConfirm: async () => {
          try {
            await deleteAgentMutation.mutateAsync(id);
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
          totalLabel={t("agents.count", { count: agentEntries.length })}
          counts={enabledCounts}
          appIds={MCP_SKILLS_APP_IDS}
        />

        <div className="flex-1 overflow-y-auto overflow-x-hidden pb-24">
          {isLoading ? (
            <div className="text-center py-12 text-muted-foreground">
              {t("common.loading")}
            </div>
          ) : agentEntries.length === 0 ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 mx-auto mb-4 bg-muted rounded-full flex items-center justify-center">
                <Bot size={24} className="text-muted-foreground" />
              </div>
              <h3 className="text-lg font-medium text-foreground mb-2">
                {t("agents.empty")}
              </h3>
              <p className="text-muted-foreground text-sm">
                {t("agents.emptyDescription")}
              </p>
            </div>
          ) : (
            <TooltipProvider delayDuration={300}>
              <div className="rounded-xl border border-border-default overflow-hidden">
                {agentEntries.map(([id, agent], index) => (
                  <AgentListItem
                    key={id}
                    id={id}
                    agent={agent}
                    onToggleApp={handleToggleApp}
                    onEdit={handleEdit}
                    onDelete={handleDelete}
                    isLast={index === agentEntries.length - 1}
                  />
                ))}
              </div>
            </TooltipProvider>
          )}
        </div>

        {isFormOpen && (
          <AgentFormPanel
            editingId={editingId ?? undefined}
            initialData={
              editingId && agentsMap ? agentsMap[editingId] : undefined
            }
            existingIds={agentsMap ? Object.keys(agentsMap) : []}
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

AgentsPanel.displayName = "AgentsPanel";

export { AgentsPanel };

// ============================================================================
// List Item
// ============================================================================

interface AgentListItemProps {
  id: string;
  agent: AgentDefinition;
  onToggleApp: (agentId: string, app: AppId, enabled: boolean) => void;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  isLast?: boolean;
}

const AgentListItem: React.FC<AgentListItemProps> = ({
  id,
  agent,
  onToggleApp,
  onEdit,
  onDelete,
  isLast,
}) => {
  const { t } = useTranslation();
  const name = agent.name || id;
  const description = agent.description || "";

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
        apps={toAppRecord(agent.apps)}
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

// ============================================================================
// Form Panel
// ============================================================================

interface AgentFormPanelProps {
  editingId?: string;
  initialData?: AgentDefinition;
  existingIds: string[];
  onClose: () => void;
}

const AgentFormPanel: React.FC<AgentFormPanelProps> = ({
  editingId,
  initialData,
  existingIds,
  onClose,
}) => {
  const { t } = useTranslation();
  const upsertMutation = useUpsertAgent();
  const isEditing = !!editingId;

  const [formId, setFormId] = useState(() => editingId || initialData?.id || "");
  const [formName, setFormName] = useState(initialData?.name || "");
  const [formDescription, setFormDescription] = useState(
    initialData?.description || "",
  );
  const [formContent, setFormContent] = useState(initialData?.content || "");
  const [idManuallyEdited, setIdManuallyEdited] = useState(isEditing);
  const [isDarkMode, setIsDarkMode] = useState(false);

  useEffect(() => {
    setIsDarkMode(document.documentElement.classList.contains("dark"));
    const observer = new MutationObserver(() => {
      setIsDarkMode(document.documentElement.classList.contains("dark"));
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });
    return () => observer.disconnect();
  }, []);

  const [enabledApps, setEnabledApps] = useState({
    claude: initialData?.apps.claude ?? false,
    codex: initialData?.apps.codex ?? false,
    gemini: initialData?.apps.gemini ?? false,
    opencode: initialData?.apps.opencode ?? false,
  });

  const [errors, setErrors] = useState<{
    id?: string;
    name?: string;
    content?: string;
  }>({});

  // Auto-generate id from name when not editing and not manually changed
  const handleNameChange = (value: string) => {
    setFormName(value);
    if (!idManuallyEdited) {
      setFormId(toSlug(value));
    }
  };

  const handleIdChange = (value: string) => {
    setFormId(value);
    setIdManuallyEdited(true);
  };

  const validate = (): boolean => {
    const newErrors: typeof errors = {};
    if (!formId.trim()) {
      newErrors.id = t("agents.requiredId");
    } else if (!isEditing && existingIds.includes(formId.trim())) {
      newErrors.id = t("agents.duplicateId", { id: formId });
    }
    if (!formName.trim()) newErrors.name = t("agents.requiredName");
    if (!formContent.trim()) newErrors.content = t("agents.requiredContent");
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSave = async () => {
    if (!validate()) return;

    const now = Date.now();
    const agent: AgentDefinition = {
      id: formId.trim(),
      name: formName.trim(),
      content: formContent,
      description: formDescription.trim() || undefined,
      apps: enabledApps,
      createdAt: initialData?.createdAt ?? now,
      updatedAt: now,
    };

    try {
      await upsertMutation.mutateAsync(agent);
      toast.success(t("agents.saveSuccess"), { closeButton: true });
      onClose();
    } catch (error) {
      toast.error(t("agents.saveFailed"), { description: String(error) });
    }
  };

  const formTitle = isEditing
    ? t("agents.editAgent")
    : t("agents.addAgent");

  return (
    <FullScreenPanel
      isOpen
      title={formTitle}
      onClose={onClose}
      footer={
        <>
          <Button variant="outline" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button
            onClick={handleSave}
            disabled={upsertMutation.isPending}
            className="gap-2"
          >
            <Save size={14} />
            {upsertMutation.isPending ? t("common.saving") : t("common.save")}
          </Button>
        </>
      }
    >
      <div className="space-y-5 max-w-2xl">
        {/* Profile ID */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium text-foreground">
            {t("agents.profileId")}
          </label>
          <Input
            value={formId}
            onChange={(e) => handleIdChange(e.target.value)}
            placeholder={t("agents.profileIdPlaceholder")}
            disabled={isEditing}
            className={errors.id ? "border-red-500" : ""}
          />
          {errors.id && (
            <p className="text-xs text-red-500">{errors.id}</p>
          )}
        </div>

        {/* Name */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium text-foreground">
            {t("agents.profileName")}
          </label>
          <Input
            value={formName}
            onChange={(e) => handleNameChange(e.target.value)}
            placeholder={t("agents.profileNamePlaceholder")}
            className={errors.name ? "border-red-500" : ""}
          />
          {errors.name && (
            <p className="text-xs text-red-500">{errors.name}</p>
          )}
        </div>

        {/* Description */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium text-foreground">
            {t("agents.description")}
            <span className="text-muted-foreground ml-1 font-normal text-xs">
              ({t("common.auto", { defaultValue: "optional" })})
            </span>
          </label>
          <Input
            value={formDescription}
            onChange={(e) => setFormDescription(e.target.value)}
            placeholder={t("agents.descriptionPlaceholder")}
          />
        </div>

        {/* Target Apps */}
        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">
            {t("agents.targetApps")}
          </label>
          <div className="flex items-center gap-4 flex-wrap">
            {MCP_SKILLS_APP_IDS.map((app) => (
              <label
                key={app}
                className="flex items-center gap-2 cursor-pointer select-none text-sm"
              >
                <Checkbox
                  checked={enabledApps[app as AgentAppId]}
                  onCheckedChange={(checked) =>
                    setEnabledApps((prev) => ({
                      ...prev,
                      [app]: !!checked,
                    }))
                  }
                />
                <span className="capitalize">{app}</span>
              </label>
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium text-foreground">
            {t("agents.content")}
          </label>
          <MarkdownEditor
            value={formContent}
            onChange={setFormContent}
            placeholder={t("agents.contentPlaceholder")}
            darkMode={isDarkMode}
            minHeight="300px"
            className={
              errors.content ? "border border-red-500 rounded-md" : ""
            }
          />
          {errors.content && (
            <p className="text-xs text-red-500">{errors.content}</p>
          )}
        </div>
      </div>
    </FullScreenPanel>
  );
};
