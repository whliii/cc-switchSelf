use indexmap::IndexMap;

use crate::app_config::AppType;
use crate::config::write_text_file;
use crate::error::AppError;
use crate::prompt::{Prompt, PromptApps};
use crate::prompt_files::prompt_file_path;
use crate::store::AppState;

/// 安全地获取当前 Unix 时间戳
fn get_unix_timestamp() -> Result<i64, AppError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|e| AppError::Message(format!("Failed to get system time: {e}")))
}

/// 将 AppType 映射到数据库列名
fn app_to_col(app: &AppType) -> &'static str {
    match app {
        AppType::Claude => "claude_enabled",
        AppType::Codex => "codex_enabled",
        AppType::Gemini => "gemini_enabled",
        AppType::OpenCode | AppType::OpenClaw => "opencode_enabled",
    }
}

/// 读取 prompt 的 app 启用状态
fn app_enabled(apps: &PromptApps, app: &AppType) -> bool {
    match app {
        AppType::Claude => apps.claude,
        AppType::Codex => apps.codex,
        AppType::Gemini => apps.gemini,
        AppType::OpenCode | AppType::OpenClaw => apps.opencode,
    }
}

/// 写入 app 的提示词文件，若内容为空则清空文件
fn sync_app_file(app: &AppType, content: Option<&str>) -> Result<(), AppError> {
    let path = prompt_file_path(app)?;
    let text = content.unwrap_or("");
    write_text_file(&path, text)
}

pub struct PromptService;

impl PromptService {
    /// 获取所有提示词（全局）
    pub fn get_prompts(state: &AppState) -> Result<IndexMap<String, Prompt>, AppError> {
        state.db.get_prompts()
    }

    /// 新增或更新提示词
    ///
    /// 保存后，对每个 app 检查新数据中的 enabled 标志：
    /// - 若 enabled=true，写入对应 app 文件
    /// - 若 enabled=false，且该 app 现在没有任何启用提示词，清空文件
    pub fn upsert_prompt(state: &AppState, prompt: Prompt) -> Result<(), AppError> {
        let new_apps = prompt.apps.clone();
        state.db.save_prompt(&prompt)?;

        let all_prompts = state.db.get_prompts()?;
        let apps = [
            AppType::Claude,
            AppType::Codex,
            AppType::Gemini,
            AppType::OpenCode,
        ];
        for app in &apps {
            if app_enabled(&new_apps, app) {
                sync_app_file(app, Some(&prompt.content))?;
            } else {
                // 检查是否还有其他启用的提示词
                let still_enabled = all_prompts
                    .values()
                    .any(|p| p.id != prompt.id && app_enabled(&p.apps, app));
                if !still_enabled {
                    // 若刚保存的也已禁用，确认再清空
                    let just_saved_enabled = all_prompts
                        .get(&prompt.id)
                        .map(|p| app_enabled(&p.apps, app))
                        .unwrap_or(false);
                    if !just_saved_enabled {
                        let path = prompt_file_path(app)?;
                        if path.exists() {
                            let _ = write_text_file(&path, "");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 删除提示词
    ///
    /// 若该提示词在某个 app 中处于启用状态，删除后清空对应 app 文件。
    pub fn delete_prompt(state: &AppState, id: &str) -> Result<(), AppError> {
        // 先读出当前状态，以便删除后清理文件
        let prompts = state.db.get_prompts()?;
        let target = prompts.get(id).cloned();

        state.db.delete_prompt(id)?;

        if let Some(prompt) = target {
            let apps = [
                AppType::Claude,
                AppType::Codex,
                AppType::Gemini,
                AppType::OpenCode,
            ];
            for app in &apps {
                if app_enabled(&prompt.apps, app) {
                    // 被删除的是该 app 的活跃提示词，清空文件
                    let path = prompt_file_path(app)?;
                    if path.exists() {
                        let _ = write_text_file(&path, "");
                    }
                }
            }
        }
        Ok(())
    }

    /// 切换提示词对指定 app 的启用状态（互斥）
    pub fn toggle_prompt_app(
        state: &AppState,
        id: &str,
        app: AppType,
        enabled: bool,
    ) -> Result<(), AppError> {
        let col = app_to_col(&app);
        state.db.toggle_prompt_app(id, col, enabled)?;

        // 同步文件
        if enabled {
            // 写入被启用提示词的内容
            let prompts = state.db.get_prompts()?;
            if let Some(prompt) = prompts.get(id) {
                sync_app_file(&app, Some(&prompt.content))?;
            }
        } else {
            // 检查是否还有其他启用的提示词
            let prompts = state.db.get_prompts()?;
            let any_enabled = prompts.values().any(|p| app_enabled(&p.apps, &app));
            if !any_enabled {
                let path = prompt_file_path(&app)?;
                if path.exists() {
                    let _ = write_text_file(&path, "");
                }
            }
        }
        Ok(())
    }

    /// 从文件导入提示词
    pub fn import_from_file(state: &AppState, app: AppType) -> Result<String, AppError> {
        let file_path = prompt_file_path(&app)?;

        if !file_path.exists() {
            return Err(AppError::Message("提示词文件不存在".to_string()));
        }

        let content =
            std::fs::read_to_string(&file_path).map_err(|e| AppError::io(&file_path, e))?;
        let timestamp = get_unix_timestamp()?;

        let id = format!("imported-{timestamp}");
        let prompt = Prompt {
            id: id.clone(),
            name: format!(
                "导入的提示词 {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            ),
            content,
            description: Some("从现有配置文件导入".to_string()),
            apps: PromptApps::default(),
            created_at: Some(timestamp),
            updated_at: Some(timestamp),
        };

        Self::upsert_prompt(state, prompt)?;
        Ok(id)
    }

    pub fn get_current_file_content(app: AppType) -> Result<Option<String>, AppError> {
        let file_path = prompt_file_path(&app)?;
        if !file_path.exists() {
            return Ok(None);
        }
        let content =
            std::fs::read_to_string(&file_path).map_err(|e| AppError::io(&file_path, e))?;
        Ok(Some(content))
    }

    /// 首次启动时从现有提示词文件自动导入（如果存在）
    /// 检查是否已有该 app 启用的提示词，无则导入并置对应 app_enabled=true
    pub fn import_from_file_on_first_launch(
        state: &AppState,
        app: AppType,
    ) -> Result<usize, AppError> {
        // 幂等性保护：该 app 已有启用的提示词则跳过
        let existing = state.db.get_prompts()?;
        let already_enabled = existing.values().any(|p| app_enabled(&p.apps, &app));
        if already_enabled {
            return Ok(0);
        }

        let file_path = prompt_file_path(&app)?;
        if !file_path.exists() {
            return Ok(0);
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("读取提示词文件失败: {file_path:?}, 错误: {e}");
                return Ok(0);
            }
        };

        if content.trim().is_empty() {
            return Ok(0);
        }

        log::info!("发现提示词文件，自动导入: {file_path:?}");

        let timestamp = get_unix_timestamp()?;
        let id = format!("auto-imported-{timestamp}");

        // 构建 apps，只启用当前 app
        let mut apps = PromptApps::default();
        match app {
            AppType::Claude => apps.claude = true,
            AppType::Codex => apps.codex = true,
            AppType::Gemini => apps.gemini = true,
            AppType::OpenCode | AppType::OpenClaw => apps.opencode = true,
        }

        let prompt = Prompt {
            id: id.clone(),
            name: format!(
                "Auto-imported Prompt {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            ),
            content,
            description: Some("Automatically imported on first launch".to_string()),
            apps,
            created_at: Some(timestamp),
            updated_at: Some(timestamp),
        };

        state.db.save_prompt(&prompt)?;

        log::info!("自动导入完成: {}", app.as_str());
        Ok(1)
    }
}
