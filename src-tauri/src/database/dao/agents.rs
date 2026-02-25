//! Agent 定义数据访问对象
//!
//! 提供 agent_definitions 表的 CRUD 操作。

use crate::agent::AgentDefinition;
use crate::app_config::McpApps;
use crate::database::{lock_conn, Database};
use crate::error::AppError;
use indexmap::IndexMap;
use rusqlite::params;

impl Database {
    /// 获取所有 Agent 定义（按 created_at ASC, id ASC 排序）
    pub fn get_all_agents(&self) -> Result<IndexMap<String, AgentDefinition>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn.prepare(
            "SELECT id, name, content, description,
                    enabled_claude, enabled_codex, enabled_gemini, enabled_opencode,
                    created_at, updated_at
             FROM agent_definitions
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        let agent_iter = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let content: String = row.get(2)?;
                let description: Option<String> = row.get(3)?;
                let enabled_claude: bool = row.get(4)?;
                let enabled_codex: bool = row.get(5)?;
                let enabled_gemini: bool = row.get(6)?;
                let enabled_opencode: bool = row.get(7)?;
                let created_at: Option<i64> = row.get(8)?;
                let updated_at: Option<i64> = row.get(9)?;

                Ok((
                    id.clone(),
                    AgentDefinition {
                        id,
                        name,
                        content,
                        description,
                        apps: McpApps {
                            claude: enabled_claude,
                            codex: enabled_codex,
                            gemini: enabled_gemini,
                            opencode: enabled_opencode,
                        },
                        created_at,
                        updated_at,
                    },
                ))
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut agents = IndexMap::new();
        for agent_res in agent_iter {
            let (id, agent) = agent_res.map_err(|e| AppError::Database(e.to_string()))?;
            agents.insert(id, agent);
        }
        Ok(agents)
    }

    /// 按 id 查询单个 Agent（避免全表扫描）
    pub fn get_agent_by_id(&self, id: &str) -> Result<Option<AgentDefinition>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn.prepare(
            "SELECT id, name, content, description,
                    enabled_claude, enabled_codex, enabled_gemini, enabled_opencode,
                    created_at, updated_at
             FROM agent_definitions
             WHERE id = ?1",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            let agent_id: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
            let name: String = row.get(1).map_err(|e| AppError::Database(e.to_string()))?;
            let content: String = row.get(2).map_err(|e| AppError::Database(e.to_string()))?;
            let description: Option<String> =
                row.get(3).map_err(|e| AppError::Database(e.to_string()))?;
            let enabled_claude: bool =
                row.get(4).map_err(|e| AppError::Database(e.to_string()))?;
            let enabled_codex: bool =
                row.get(5).map_err(|e| AppError::Database(e.to_string()))?;
            let enabled_gemini: bool =
                row.get(6).map_err(|e| AppError::Database(e.to_string()))?;
            let enabled_opencode: bool =
                row.get(7).map_err(|e| AppError::Database(e.to_string()))?;
            let created_at: Option<i64> =
                row.get(8).map_err(|e| AppError::Database(e.to_string()))?;
            let updated_at: Option<i64> =
                row.get(9).map_err(|e| AppError::Database(e.to_string()))?;

            Ok(Some(AgentDefinition {
                id: agent_id,
                name,
                content,
                description,
                apps: McpApps {
                    claude: enabled_claude,
                    codex: enabled_codex,
                    gemini: enabled_gemini,
                    opencode: enabled_opencode,
                },
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// 保存（新增或替换）Agent 定义
    pub fn save_agent(&self, agent: &AgentDefinition) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO agent_definitions (
                id, name, content, description,
                enabled_claude, enabled_codex, enabled_gemini, enabled_opencode,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                agent.id,
                agent.name,
                agent.content,
                agent.description,
                agent.apps.claude,
                agent.apps.codex,
                agent.apps.gemini,
                agent.apps.opencode,
                agent.created_at,
                agent.updated_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 删除 Agent 定义
    pub fn delete_agent(&self, id: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "DELETE FROM agent_definitions WHERE id = ?1",
            params![id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
