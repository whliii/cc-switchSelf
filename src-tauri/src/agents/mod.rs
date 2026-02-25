//! Agent 文件同步模块
//!
//! 负责将 AgentDefinition 写入各 CLI 工具对应的 agent 文件，
//! 或从各工具文件中移除指定 agent。
//!
//! ## 各工具写入规范
//!
//! | 工具      | 路径                                    | 格式                              |
//! |-----------|----------------------------------------|-----------------------------------|
//! | Claude    | `~/.claude/agents/{id}.md`             | YAML frontmatter + Markdown body  |
//! | OpenCode  | `~/.config/opencode/agents/{id}.md`    | YAML frontmatter + Markdown body  |
//! | Codex     | `~/.codex/AGENTS.md`                   | cc-switch marker 分区块            |
//! | Gemini    | `~/.gemini/GEMINI.md`                  | cc-switch marker 分区块            |

mod claude;
mod codex;
mod gemini;
mod opencode;

use crate::agent::AgentDefinition;
use crate::app_config::AppType;
use crate::error::AppError;

/// 将 Agent 同步到指定工具
pub fn sync_agent_to_app(agent: &AgentDefinition, app: &AppType) -> Result<(), AppError> {
    match app {
        AppType::Claude => claude::write_agent(agent),
        AppType::Codex => codex::write_agent(agent),
        AppType::Gemini => gemini::write_agent(agent),
        AppType::OpenCode => opencode::write_agent(agent),
        AppType::OpenClaw => {
            log::debug!("OpenClaw agent sync not supported, skipping");
            Ok(())
        }
    }
}

/// 从指定工具中移除 Agent
pub fn remove_agent_from_app(id: &str, app: &AppType) -> Result<(), AppError> {
    match app {
        AppType::Claude => claude::remove_agent(id),
        AppType::Codex => codex::remove_agent(id),
        AppType::Gemini => gemini::remove_agent(id),
        AppType::OpenCode => opencode::remove_agent(id),
        AppType::OpenClaw => {
            log::debug!("OpenClaw agent remove not supported, skipping");
            Ok(())
        }
    }
}
