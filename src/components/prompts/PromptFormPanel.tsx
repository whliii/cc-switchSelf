import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Save } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import MarkdownEditor from "@/components/MarkdownEditor";
import { FullScreenPanel } from "@/components/common/FullScreenPanel";
import type { Prompt, PromptApps } from "@/lib/api/prompts";
import { useUpsertPrompt } from "@/hooks/usePrompts";
import { MCP_SKILLS_APP_IDS } from "@/config/appConfig";

interface PromptFormPanelProps {
  editingId?: string;
  initialData?: Prompt;
  onClose: () => void;
}

const PromptFormPanel: React.FC<PromptFormPanelProps> = ({
  editingId,
  initialData,
  onClose,
}) => {
  const { t } = useTranslation();
  const upsertMutation = useUpsertPrompt();
  const isEditing = !!editingId;

  const [name, setName] = useState(initialData?.name || "");
  const [description, setDescription] = useState(initialData?.description || "");
  const [content, setContent] = useState(initialData?.content || "");
  const [isDarkMode, setIsDarkMode] = useState(false);
  const [enabledApps, setEnabledApps] = useState<PromptApps>({
    claude: initialData?.apps.claude ?? false,
    codex: initialData?.apps.codex ?? false,
    gemini: initialData?.apps.gemini ?? false,
    opencode: initialData?.apps.opencode ?? false,
  });

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

  const handleSave = async () => {
    if (!name.trim()) return;

    const now = Math.floor(Date.now() / 1000);
    const prompt: Prompt = {
      id: editingId || `prompt-${Date.now()}`,
      name: name.trim(),
      description: description.trim() || undefined,
      content: content.trim(),
      apps: enabledApps,
      createdAt: initialData?.createdAt ?? now,
      updatedAt: now,
    };

    try {
      await upsertMutation.mutateAsync(prompt);
      toast.success(t("prompts.saveSuccess"), { closeButton: true });
      onClose();
    } catch (error) {
      toast.error(t("prompts.saveFailed"), { description: String(error) });
    }
  };

  const title = isEditing ? t("prompts.editTitle") : t("prompts.addTitle");

  return (
    <FullScreenPanel
      isOpen
      title={title}
      onClose={onClose}
      footer={
        <>
          <Button variant="outline" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button
            onClick={handleSave}
            disabled={!name.trim() || upsertMutation.isPending}
            className="gap-2"
          >
            <Save size={14} />
            {upsertMutation.isPending ? t("common.saving") : t("common.save")}
          </Button>
        </>
      }
    >
      <div className="space-y-5 max-w-2xl">
        {/* Name */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium text-foreground">
            {t("prompts.name")}
          </label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t("prompts.namePlaceholder")}
          />
        </div>

        {/* Description */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium text-foreground">
            {t("prompts.description")}
            <span className="text-muted-foreground ml-1 font-normal text-xs">
              ({t("common.auto", { defaultValue: "optional" })})
            </span>
          </label>
          <Input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t("prompts.descriptionPlaceholder")}
          />
        </div>

        {/* Target Apps */}
        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">
            {t("prompts.targetApps", { defaultValue: "Target Apps" })}
          </label>
          <div className="flex items-center gap-4 flex-wrap">
            {MCP_SKILLS_APP_IDS.map((app) => (
              <label
                key={app}
                className="flex items-center gap-2 cursor-pointer select-none text-sm"
              >
                <Checkbox
                  checked={enabledApps[app as keyof PromptApps]}
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
            {t("prompts.content")}
          </label>
          <MarkdownEditor
            value={content}
            onChange={setContent}
            placeholder={t("prompts.contentPlaceholder", { filename: "CLAUDE.md" })}
            darkMode={isDarkMode}
            minHeight="300px"
          />
        </div>
      </div>
    </FullScreenPanel>
  );
};

export default PromptFormPanel;
