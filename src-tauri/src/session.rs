use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// 单条消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: String,
}

/// 完整会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<Message>,
}

/// 会话摘要（列表用，不含消息内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
}

/// 当前时间字符串
fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 会话存储目录
fn sessions_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("无法获取应用数据目录");
    let sessions_dir = dir.join("sessions");
    std::fs::create_dir_all(&sessions_dir).ok();
    sessions_dir
}

/// 生成唯一 ID
fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// 获取会话文件路径
fn session_path(app: &tauri::AppHandle, id: &str) -> PathBuf {
    sessions_dir(app).join(format!("{}.json", id))
}

/// 列出所有会话摘要
pub fn list_sessions(app: &tauri::AppHandle) -> Vec<SessionSummary> {
    let dir = sessions_dir(app);
    let mut sessions: Vec<SessionSummary> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&json) {
                        let count = session.messages.len();
                        sessions.push(SessionSummary {
                            id: session.id,
                            name: session.name,
                            created_at: session.created_at,
                            updated_at: session.updated_at,
                            message_count: count,
                        });
                    }
                }
            }
        }
    }

    // 按更新时间降序（最新的在前）
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    sessions
}

/// 创建新会话
pub fn create_session(app: &tauri::AppHandle) -> Session {
    let now = now();
    let session = Session {
        id: new_id(),
        name: format!("新对话 {}", &now[..10]),
        created_at: now.clone(),
        updated_at: now,
        messages: Vec::new(),
    };

    // 立即持久化
    let path = session_path(app, &session.id);
    if let Ok(json) = serde_json::to_string_pretty(&session) {
        std::fs::write(&path, json).ok();
    }

    session
}

/// 获取单个会话（含消息）
pub fn get_session(app: &tauri::AppHandle, id: &str) -> Option<Session> {
    let path = session_path(app, id);
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

/// 删除会话
pub fn delete_session(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let path = session_path(app, id);
    std::fs::remove_file(&path).map_err(|e| format!("删除会话失败: {}", e))
}

/// 重命名会话
pub fn rename_session(app: &tauri::AppHandle, id: &str, name: &str) -> Result<(), String> {
    let mut session = get_session(app, id).ok_or("会话不存在")?;
    session.name = name.to_string();
    session.updated_at = now();

    let path = session_path(app, id);
    let json = serde_json::to_string_pretty(&session).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))
}

/// 保存会话（完整覆盖）
#[allow(dead_code)]
pub fn save_session(app: &tauri::AppHandle, session: &Session) -> Result<(), String> {
    let path = session_path(app, &session.id);
    let json = serde_json::to_string_pretty(session).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))
}
