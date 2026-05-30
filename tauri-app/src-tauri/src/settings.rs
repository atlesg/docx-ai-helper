use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub openai_api_key: String,
    pub gemini_api_key: String,
    pub active_provider: String, // "openai" | "gemini"
    pub openai_model: String,     // e.g., "gpt-4o", "gpt-4o-mini"
    pub gemini_model: String,     // e.g., "gemini-1.5-pro", "gemini-1.5-flash"
    pub port: u16,                // defaults to 44320
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openai_api_key: String::new(),
            gemini_api_key: String::new(),
            active_provider: "openai".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            gemini_model: "gemini-1.5-flash".to_string(),
            port: 44320,
        }
    }
}

fn get_settings_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    // Ensure the directory exists
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    
    path.push("settings.json");
    Ok(path)
}

pub fn load_settings(app_handle: &tauri::AppHandle) -> AppSettings {
    match get_settings_path(app_handle) {
        Ok(path) => {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                        return settings;
                    }
                }
            }
            AppSettings::default()
        }
        Err(_) => AppSettings::default(),
    }
}

pub fn save_settings(app_handle: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app_handle)?;
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
