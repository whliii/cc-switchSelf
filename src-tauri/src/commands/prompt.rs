use indexmap::IndexMap;
use std::str::FromStr;

use tauri::State;

use crate::app_config::AppType;
use crate::prompt::Prompt;
use crate::services::PromptService;
use crate::store::AppState;

#[tauri::command]
pub async fn get_prompts(
    state: State<'_, AppState>,
) -> Result<IndexMap<String, Prompt>, String> {
    PromptService::get_prompts(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upsert_prompt(
    prompt: Prompt,
    state: State<'_, AppState>,
) -> Result<(), String> {
    PromptService::upsert_prompt(&state, prompt).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_prompt(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    PromptService::delete_prompt(&state, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_prompt_app(
    id: String,
    app: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    PromptService::toggle_prompt_app(&state, &id, app_type, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_prompt_from_file(
    app: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    PromptService::import_from_file(&state, app_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_prompt_file_content(app: String) -> Result<Option<String>, String> {
    let app_type = AppType::from_str(&app).map_err(|e| e.to_string())?;
    PromptService::get_current_file_content(app_type).map_err(|e| e.to_string())
}
