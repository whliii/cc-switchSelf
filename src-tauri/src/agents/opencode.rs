//! OpenCode agent 文件同步
//!
//! 写入路径：`~/.config/opencode/agents/{id}.md`
//! 格式：YAML frontmatter（name, description）+ Markdown body（content）

use crate::agent::AgentDefinition;
use crate::config::write_text_file;
use crate::error::AppError;
use crate::opencode_config::get_opencode_dir;
use std::path::PathBuf;

fn agent_path(id: &str) -> PathBuf {
    get_opencode_dir().join("agents").join(format!("{id}.md"))
}

/// 写入 `~/.config/opencode/agents/{id}.md`
pub fn write_agent(agent: &AgentDefinition) -> Result<(), AppError> {
    let path = agent_path(&agent.id);
    let content = build_frontmatter_md(agent);
    write_text_file(&path, &content)
}

/// 删除 `~/.config/opencode/agents/{id}.md`（不存在时静默忽略）
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
    if !fm.ends_with('\n') {
        fm.push('\n');
    }
    fm
}
