//! Gemini agent 文件同步
//!
//! 写入路径：`~/.gemini/GEMINI.md`（共享文件，每个 agent 占一个 marker 区块）
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
use crate::config::write_text_file;
use crate::error::AppError;
use crate::gemini_config::get_gemini_dir;
use std::path::PathBuf;

fn agents_file_path() -> PathBuf {
    get_gemini_dir().join("GEMINI.md")
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

/// Upsert agent 区块到 `~/.gemini/GEMINI.md`
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

/// 从 `~/.gemini/GEMINI.md` 中删除指定 agent 区块
pub fn remove_agent(id: &str) -> Result<(), AppError> {
    let path = agents_file_path();
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    let new_content = remove_block(&content, id);
    write_text_file(&path, &new_content)
}

fn upsert_block(content: &str, agent: &AgentDefinition) -> String {
    let start = start_marker(&agent.id);
    let end = end_marker(&agent.id);
    let new_block = build_block(agent);

    if let (Some(start_pos), Some(end_pos)) = (content.find(&start), content.find(&end)) {
        let after_end = end_pos + end.len();
        let after_end = if content[after_end..].starts_with('\n') {
            after_end + 1
        } else {
            after_end
        };
        format!("{}{}{}", &content[..start_pos], new_block, &content[after_end..])
    } else {
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

fn remove_block(content: &str, id: &str) -> String {
    let start = start_marker(id);
    let end = end_marker(id);

    if let (Some(start_pos), Some(end_pos)) = (content.find(&start), content.find(&end)) {
        let after_end = end_pos + end.len();
        let after_end = if content[after_end..].starts_with('\n') {
            after_end + 1
        } else {
            after_end
        };
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
