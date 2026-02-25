//! Codex agent 文件同步
//!
//! 写入路径：`~/.codex/AGENTS.md`（共享文件，每个 agent 占一个 marker 区块）
//!
//! 区块格式：
//! ```text
//! <!-- cc-switch:agent:{id} -->
//! # {name}
//!
//! {content}
//!
//! <!-- /cc-switch:agent:{id} -->
//! ```

use crate::agent::AgentDefinition;
use crate::codex_config::get_codex_config_dir;
use crate::config::write_text_file;
use crate::error::AppError;
use std::path::PathBuf;

fn agents_file_path() -> PathBuf {
    get_codex_config_dir().join("AGENTS.md")
}

fn start_marker(id: &str) -> String {
    format!("<!-- cc-switch:agent:{id} -->")
}

fn end_marker(id: &str) -> String {
    format!("<!-- /cc-switch:agent:{id} -->")
}

fn build_block(agent: &AgentDefinition) -> String {
    let mut block = String::new();
    block.push_str(&start_marker(&agent.id));
    block.push('\n');
    block.push_str(&format!("# {}\n", agent.name));
    block.push('\n');
    block.push_str(&agent.content);
    if !block.ends_with('\n') {
        block.push('\n');
    }
    block.push('\n');
    block.push_str(&end_marker(&agent.id));
    block.push('\n');
    block
}

/// Upsert agent 区块到 `~/.codex/AGENTS.md`
pub fn write_agent(agent: &AgentDefinition) -> Result<(), AppError> {
    let path = agents_file_path();
    let existing = if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?
    } else {
        String::new()
    };

    let new_content = upsert_block(&existing, agent);
    write_text_file(&path, &new_content)
}

/// 从 `~/.codex/AGENTS.md` 中删除指定 agent 区块
pub fn remove_agent(id: &str) -> Result<(), AppError> {
    let path = agents_file_path();
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    let new_content = remove_block(&content, id);
    write_text_file(&path, &new_content)
}

/// 在文件内容中 upsert 指定 agent 的区块
fn upsert_block(content: &str, agent: &AgentDefinition) -> String {
    let start = start_marker(&agent.id);
    let end = end_marker(&agent.id);
    let new_block = build_block(agent);

    if let (Some(start_pos), Some(end_pos)) = (content.find(&start), content.find(&end)) {
        // 区块已存在：替换
        let after_end = end_pos + end.len();
        // 跳过末尾的换行
        let after_end = if content[after_end..].starts_with('\n') {
            after_end + 1
        } else {
            after_end
        };
        format!("{}{}{}", &content[..start_pos], new_block, &content[after_end..])
    } else {
        // 区块不存在：追加
        let mut result = content.to_string();
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        if !result.is_empty() && !result.ends_with("\n\n") {
            result.push('\n');
        }
        result.push_str(&new_block);
        result
    }
}

/// 从文件内容中删除指定 agent 的区块
fn remove_block(content: &str, id: &str) -> String {
    let start = start_marker(id);
    let end = end_marker(id);

    if let (Some(start_pos), Some(end_pos)) = (content.find(&start), content.find(&end)) {
        let after_end = end_pos + end.len();
        // 跳过末尾的换行
        let after_end = if content[after_end..].starts_with('\n') {
            after_end + 1
        } else {
            after_end
        };
        // 如果区块前面有额外的空行，也一并删除
        let start_pos = if start_pos > 0 && content[..start_pos].ends_with("\n\n") {
            start_pos - 1
        } else {
            start_pos
        };
        format!("{}{}", &content[..start_pos], &content[after_end..])
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::McpApps;

    fn make_agent(id: &str, name: &str, content: &str) -> AgentDefinition {
        AgentDefinition {
            id: id.to_string(),
            name: name.to_string(),
            content: content.to_string(),
            description: None,
            apps: McpApps::default(),
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_upsert_empty_file() {
        let agent = make_agent("test-agent", "Test Agent", "You are a test agent.");
        let result = upsert_block("", &agent);
        assert!(result.contains("<!-- cc-switch:agent:test-agent -->"));
        assert!(result.contains("<!-- /cc-switch:agent:test-agent -->"));
        assert!(result.contains("# Test Agent"));
        assert!(result.contains("You are a test agent."));
    }

    #[test]
    fn test_upsert_existing_block() {
        let agent = make_agent("test-agent", "Test Agent", "Initial content.");
        let initial = upsert_block("", &agent);

        let agent2 = make_agent("test-agent", "Test Agent", "Updated content.");
        let result = upsert_block(&initial, &agent2);
        assert!(result.contains("Updated content."));
        assert!(!result.contains("Initial content."));
        // Should only have one block
        assert_eq!(result.matches("<!-- cc-switch:agent:test-agent -->").count(), 1);
    }

    #[test]
    fn test_remove_block() {
        let agent = make_agent("test-agent", "Test Agent", "Some content.");
        let content = upsert_block("", &agent);
        let result = remove_block(&content, "test-agent");
        assert!(!result.contains("cc-switch:agent:test-agent"));
    }

    #[test]
    fn test_remove_nonexistent_block() {
        let content = "Some existing content\n";
        let result = remove_block(content, "nonexistent");
        assert_eq!(result, content);
    }
}
