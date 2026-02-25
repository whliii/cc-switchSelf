//! Claude Code agent 文件同步
//!
//! 写入路径：`~/.claude/agents/{id}.md`
//! 格式：YAML frontmatter（name, description）+ Markdown body（content）

use crate::agent::AgentDefinition;
use crate::config::{get_claude_config_dir, write_text_file};
use crate::error::AppError;
use std::path::PathBuf;

fn agent_path(id: &str) -> PathBuf {
    get_claude_config_dir().join("agents").join(format!("{id}.md"))
}

/// 写入 `~/.claude/agents/{id}.md`
pub fn write_agent(agent: &AgentDefinition) -> Result<(), AppError> {
    let path = agent_path(&agent.id);
    let content = build_frontmatter_md(agent);
    write_text_file(&path, &content)
}

/// 删除 `~/.claude/agents/{id}.md`（不存在时静默忽略）
pub fn remove_agent(id: &str) -> Result<(), AppError> {
    let path = agent_path(id);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| AppError::io(&path, e))?;
    }
    Ok(())
}

fn build_frontmatter_md(agent: &AgentDefinition) -> String {
    let mut fm = String::from("---\n");
    fm.push_str(&format!("name: {}\n", agent.name));
    if let Some(desc) = &agent.description {
        if !desc.is_empty() {
            fm.push_str(&format!("description: {}\n", desc));
        }
    }
    fm.push_str("---\n");
    fm.push('\n');
    fm.push_str(&agent.content);
    // 确保文件末尾有换行
    if !fm.ends_with('\n') {
        fm.push('\n');
    }
    fm
}
