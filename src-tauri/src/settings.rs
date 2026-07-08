use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// 权限级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionLevel {
    #[serde(rename = "default")]
    Default,   // 默认模式：高风险操作弹窗确认
    #[serde(rename = "review")]
    Review,    // 自动审查：自动执行，审查变更
    #[serde(rename = "full")]
    Full,      // 完全访问：YOLO 模式
}

impl Default for PermissionLevel {
    fn default() -> Self {
        PermissionLevel::Default
    }
}

/// 应用设置
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

/// 获取设置文件路径
fn settings_path(app: &tauri::AppHandle) -> PathBuf {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("无法获取应用数据目录");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("settings.json")
}

/// 从系统密钥链保存 API Key
fn save_api_key_to_keyring(api_key: &str) -> Result<(), String> {
    if api_key.is_empty() {
        let entry = keyring::Entry::new("agent-pc", "deepseek-api-key")
            .map_err(|e| format!("创建密钥条目失败: {}", e))?;
        entry.delete_credential().ok();
        return Ok(());
    }

    let entry = keyring::Entry::new("agent-pc", "deepseek-api-key")
        .map_err(|e| format!("创建密钥条目失败: {}", e))?;
    entry
        .set_password(api_key)
        .map_err(|e| format!("保存密钥失败: {}", e))
}

/// 从系统密钥链读取 API Key
fn load_api_key_from_keyring() -> Option<String> {
    let entry = keyring::Entry::new("agent-pc", "deepseek-api-key").ok()?;
    entry.get_password().ok()
}

/// 保存设置到文件
pub fn save_settings(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    // 1. 将 API Key 存入系统密钥链
    save_api_key_to_keyring(&settings.api_key)?;

    // 2. 将其他设置存入 JSON 文件（API Key 字段写入空字符串）
    let mut file_settings = settings.clone();
    file_settings.api_key = String::new();

    let json = serde_json::to_string_pretty(&file_settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;

    let path = settings_path(app);
    std::fs::write(&path, json).map_err(|e| format!("写入设置文件失败: {}", e))?;

    Ok(())
}

/// 从文件加载设置
pub fn load_settings(app: &tauri::AppHandle) -> Settings {
    let path = settings_path(app);

    if !path.exists() {
        let mut settings = Settings::default();
        if let Some(key) = load_api_key_from_keyring() {
            settings.api_key = key;
        }
        return settings;
    }

    let json = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };

    let mut settings: Settings = match serde_json::from_str(&json) {
        Ok(s) => s,
        Err(_) => Settings::default(),
    };

    if let Some(key) = load_api_key_from_keyring() {
        settings.api_key = key;
    }

    settings
}
