use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionLevel {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "full")]
    Full,
}

impl Default for PermissionLevel {
    fn default() -> Self { PermissionLevel::Default }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub api_key: String,
    pub model: String,
    pub permission_level: PermissionLevel,
    pub working_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "deepseek-v4-flash".to_string(),
            permission_level: PermissionLevel::default(),
            working_dir: String::new(),
        }
    }
}

fn settings_path(app: &tauri::AppHandle) -> PathBuf {
    let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("settings.json")
}

/// 保存设置到文件（API Key 直接存 JSON）
pub fn save_settings(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    let path = settings_path(app);
    std::fs::write(&path, json).map_err(|e| format!("写入设置文件失败: {}", e))?;
    Ok(())
}

/// 从文件加载设置
pub fn load_settings(app: &tauri::AppHandle) -> Settings {
    let path = settings_path(app);
    if !path.exists() { return Settings::default(); }
    let json = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    serde_json::from_str(&json).unwrap_or_default()
}
