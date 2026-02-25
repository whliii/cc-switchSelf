//! 提示词数据访问对象
//!
//! 提供提示词（Prompt）的 CRUD 操作。

use crate::database::{lock_conn, Database};
use crate::error::AppError;
use crate::prompt::{Prompt, PromptApps};
use indexmap::IndexMap;
use rusqlite::params;

impl Database {
    /// 获取所有提示词（全局，不区分 app）
    pub fn get_prompts(&self) -> Result<IndexMap<String, Prompt>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, content, description,
                        claude_enabled, codex_enabled, gemini_enabled, opencode_enabled,
                        created_at, updated_at
                 FROM prompts
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let prompt_iter = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let content: String = row.get(2)?;
                let description: Option<String> = row.get(3)?;
                let claude: bool = row.get(4)?;
                let codex: bool = row.get(5)?;
                let gemini: bool = row.get(6)?;
                let opencode: bool = row.get(7)?;
                let created_at: Option<i64> = row.get(8)?;
                let updated_at: Option<i64> = row.get(9)?;

                Ok((
                    id.clone(),
                    Prompt {
                        id,
                        name,
                        content,
                        description,
                        apps: PromptApps {
                            claude,
                            codex,
                            gemini,
                            opencode,
                        },
                        created_at,
                        updated_at,
                    },
                ))
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut prompts = IndexMap::new();
        for prompt_res in prompt_iter {
            let (id, prompt) = prompt_res.map_err(|e| AppError::Database(e.to_string()))?;
            prompts.insert(id, prompt);
        }
        Ok(prompts)
    }

    /// 保存提示词（INSERT OR REPLACE）
    pub fn save_prompt(&self, prompt: &Prompt) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO prompts (
                id, name, content, description,
                claude_enabled, codex_enabled, gemini_enabled, opencode_enabled,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                prompt.id,
                prompt.name,
                prompt.content,
                prompt.description,
                prompt.apps.claude,
                prompt.apps.codex,
                prompt.apps.gemini,
                prompt.apps.opencode,
                prompt.created_at,
                prompt.updated_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 删除提示词（按 id）
    pub fn delete_prompt(&self, id: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 切换提示词对指定 app 的启用状态（互斥：同 app 同时只能有一个启用）
    ///
    /// - enabled=true：先清除该 app 所有提示词的启用标志，再设置目标
    /// - enabled=false：只清除目标
    pub fn toggle_prompt_app(
        &self,
        id: &str,
        app_col: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        // 校验列名安全性（防止 SQL 注入）
        let allowed = ["claude_enabled", "codex_enabled", "gemini_enabled", "opencode_enabled"];
        if !allowed.contains(&app_col) {
            return Err(AppError::InvalidInput(format!("非法的 app_col: {app_col}")));
        }

        let conn = lock_conn!(self.conn);
        if enabled {
            // 先全清，再设目标
            let clear_sql = format!("UPDATE prompts SET {app_col} = 0");
            conn.execute(&clear_sql, [])
                .map_err(|e| AppError::Database(format!("清除 {app_col} 失败: {e}")))?;
            let set_sql = format!("UPDATE prompts SET {app_col} = 1 WHERE id = ?1");
            conn.execute(&set_sql, params![id])
                .map_err(|e| AppError::Database(format!("设置 {app_col} 失败: {e}")))?;
        } else {
            let clear_sql = format!("UPDATE prompts SET {app_col} = 0 WHERE id = ?1");
            conn.execute(&clear_sql, params![id])
                .map_err(|e| AppError::Database(format!("清除 {app_col} 失败: {e}")))?;
        }
        Ok(())
    }
}
