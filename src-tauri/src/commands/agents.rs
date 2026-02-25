//! Agent 管理 Tauri 命令
//!
//! 镜像 `commands/mcp.rs`，提供前端调用的 IPC 接口。

use indexmap::IndexMap;
use std::str::FromStr;
use tauri::State;

use crate::agent::AgentDefinition;
use crate::app_config::AppType;
use crate::services::AgentsService;
use crate::store::AppState;

/// 获取所有 Agent 定义
#[tauri::command]
pub async fn get_agent_definitions(
    state: State<'_, AppState>,
) -> Result<IndexMap<String, AgentDefinition>, String> {
    AgentsService::get_all(&state).map_err(|e| e.to_string())
}

/// 新增或更新 Agent 定义
#[tauri::command]
pub async fn upsert_agent_definition(
    state: State<'_, AppState>,
    agent: AgentDefinition,
) -> Result<(), String> {
    AgentsService::upsert(&state, agent).map_err(|e| e.to_string())
}

/// 删除 Agent 定义
#[tauri::command]
pub async fn delete_agent_definition(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    AgentsService::delete(&state, &id).map_err(|e| e.to_string())
}

/// 切换 Agent 在指定工具的启用状态
#[tauri::command]
pub async fn toggle_agent_app(
    state: State<'_, AppState>,
    agent_id: String,
    app: String,
    enabled: bool,
) -> Result<(), String> {
    let app_ty = AppType::from_str(&app).map_err(|e| e.to_string())?;
    AgentsService::toggle_app(&state, &agent_id, app_ty, enabled).map_err(|e| e.to_string())
}
