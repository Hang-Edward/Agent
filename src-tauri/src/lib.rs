mod deepseek;
mod session;
mod settings;

use settings::Settings;
use tauri::Manager;
use uuid::Uuid;

/// 获取设置
#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Settings {
    settings::load_settings(&app)
}

/// 保存设置
#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    settings::save_settings(&app, &settings)
}

/// 列出会话
#[tauri::command]
fn list_sessions(app: tauri::AppHandle) -> Vec<session::SessionSummary> {
    session::list_sessions(&app)
}

/// 创建会话
#[tauri::command]
fn create_session(app: tauri::AppHandle) -> session::Session {
    session::create_session(&app)
}

/// 获取会话
#[tauri::command]
fn get_session(app: tauri::AppHandle, id: String) -> Option<session::Session> {
    session::get_session(&app, &id)
}

/// 删除会话
#[tauri::command]
fn delete_session(app: tauri::AppHandle, id: String) -> Result<(), String> {
    session::delete_session(&app, &id)
}

/// 重命名会话
#[tauri::command]
fn rename_session(app: tauri::AppHandle, id: String, name: String) -> Result<(), String> {
    session::rename_session(&app, &id, &name)
}

/// 发送消息到 DeepSeek API
#[tauri::command]
async fn send_message(
    app: tauri::AppHandle,
    session_id: String,
    content: String,
) -> Result<deepseek::ChatResult, String> {
    // 1. 加载设置（含 API Key 和模型选择）
    let settings = settings::load_settings(&app);

    // 2. 获取当前会话
    let mut session = session::get_session(&app, &session_id).ok_or("会话不存在")?;

    // 3. 保存用户消息
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let user_msg = session::Message {
        id: Uuid::new_v4().to_string(),
        role: session::MessageRole::User,
        content: content.clone(),
        created_at: now.clone(),
    };
    session.messages.push(user_msg);

    // 4. 将会话历史转为 DeepSeek 消息格式
    let deepseek_messages: Vec<deepseek::ChatMessage> = session
        .messages
        .iter()
        .map(|m| {
            let role = match m.role {
                session::MessageRole::User => deepseek::Role::User,
                session::MessageRole::Assistant => deepseek::Role::Assistant,
                session::MessageRole::System => deepseek::Role::System,
            };
            deepseek::ChatMessage {
                role,
                content: m.content.clone(),
            }
        })
        .collect();

    // 5. 调用 DeepSeek API
    let result = deepseek::chat_completion(&settings.api_key, &settings.model, &deepseek_messages).await?;

    // 6. 保存 AI 回复到会话
    let ai_msg = session::Message {
        id: Uuid::new_v4().to_string(),
        role: session::MessageRole::Assistant,
        content: result.content.clone(),
        created_at: now,
    };
    session.messages.push(ai_msg);
    session::save_session(&app, &session)?;

    // 7. 如果会话还没有自定义名称，用第一条消息生成名称
    if session.name.starts_with("新对话") && session.messages.len() <= 2 {
        let preview: String = content.chars().take(20).collect();
        let name = if content.len() > 20 { format!("{}...", preview) } else { preview };
        session::rename_session(&app, &session_id, &name).ok();
    }

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_sessions,
            create_session,
            get_session,
            delete_session,
            rename_session,
            send_message,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            std::fs::create_dir_all(&data_dir).ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动应用时发生错误");
}
