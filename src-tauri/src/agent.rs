//! Agent 定义数据类型
//!
//! 用于集中管理多工具 Agent 定义（system prompt / 角色卡）。

use crate::app_config::McpApps;
use serde::{Deserialize, Serialize};

/// Agent 定义（统一结构）
///
/// 对应数据库 `agent_definitions` 表。
/// `apps` 字段复用 [`McpApps`]，表示该 agent 已被启用到哪些 CLI 工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefinition {
    /// slug，用作文件名（不可重复）
    pub id: String,
    /// 显示名称
    pub name: String,
    /// Markdown 正文（系统提示词 / 角色卡内容）
    pub content: String,
    /// 可选描述
    pub description: Option<String>,
    /// 已启用的 CLI 工具集合
    pub apps: McpApps,
    /// 创建时间（Unix 毫秒）
    pub created_at: Option<i64>,
    /// 更新时间（Unix 毫秒）
    pub updated_at: Option<i64>,
}
