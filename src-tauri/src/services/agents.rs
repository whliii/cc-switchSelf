//! Agent 管理业务逻辑
//!
//! 镜像 `services/mcp.rs`，处理 agent 的 CRUD 和文件同步。

use indexmap::IndexMap;

use crate::agent::AgentDefinition;
use crate::agents;
use crate::app_config::AppType;
use crate::error::AppError;
use crate::store::AppState;

/// Agent 管理服务
pub struct AgentsService;

impl AgentsService {
    /// 获取所有 Agent 定义
    pub fn get_all(state: &AppState) -> Result<IndexMap<String, AgentDefinition>, AppError> {
        state.db.get_all_agents()
    }

    /// 新增或更新 Agent 定义，并将变更同步到对应工具文件
    pub fn upsert(state: &AppState, agent: AgentDefinition) -> Result<(), AppError> {
        // 读取旧状态（按 id 查询，避免全表扫描）
        let prev_apps = state
            .db
            .get_agent_by_id(&agent.id)?
            .map(|a| a.apps.clone())
            .unwrap_or_default();

        // 保存到数据库
        state.db.save_agent(&agent)?;

        // 处理禁用：旧版本启用但新版本取消时，从工具文件中移除
        if prev_apps.claude && !agent.apps.claude {
            agents::remove_agent_from_app(&agent.id, &AppType::Claude)?;
        }
        if prev_apps.codex && !agent.apps.codex {
            agents::remove_agent_from_app(&agent.id, &AppType::Codex)?;
        }
        if prev_apps.gemini && !agent.apps.gemini {
            agents::remove_agent_from_app(&agent.id, &AppType::Gemini)?;
        }
        if prev_apps.opencode && !agent.apps.opencode {
            agents::remove_agent_from_app(&agent.id, &AppType::OpenCode)?;
        }

        // 同步到所有启用的工具（内容可能已更新）
        Self::sync_agent_to_apps(&agent)?;

        Ok(())
    }

    /// 删除 Agent 定义，并从所有已启用工具中移除
    pub fn delete(state: &AppState, id: &str) -> Result<bool, AppError> {
        let agent = state.db.get_agent_by_id(id)?;

        if let Some(agent) = agent {
            state.db.delete_agent(id)?;

            // 从所有已启用的工具中移除
            for app in agent.apps.enabled_apps() {
                agents::remove_agent_from_app(id, &app)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 切换指定工具的启用状态（即时写入/删除文件）
    pub fn toggle_app(
        state: &AppState,
        agent_id: &str,
        app: AppType,
        enabled: bool,
    ) -> Result<(), AppError> {
        let agent = state.db.get_agent_by_id(agent_id)?;

        if let Some(mut agent) = agent {
            agent.apps.set_enabled_for(&app, enabled);
            state.db.save_agent(&agent)?;

            if enabled {
                agents::sync_agent_to_app(&agent, &app)?;
            } else {
                agents::remove_agent_from_app(agent_id, &app)?;
            }
        }

        Ok(())
    }

    /// 将 Agent 同步到所有已启用的工具
    fn sync_agent_to_apps(agent: &AgentDefinition) -> Result<(), AppError> {
        for app in agent.apps.enabled_apps() {
            agents::sync_agent_to_app(agent, &app)?;
        }
        Ok(())
    }
}
