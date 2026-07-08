mod agent;
mod code_index;
mod deepseek;
mod mcp;
mod mcp_server;
mod sandbox;
mod session;
mod settings;
mod skills;
mod terminal;
mod tools;

use settings::Settings;
use tauri::Manager;
use terminal::TerminalSession;

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

/// 列出所有 Skills
#[tauri::command]
fn list_skills(app: tauri::AppHandle) -> Vec<skills::SkillSummary> {
    skills::list_skills(&app)
}

/// 创建 Skill
#[tauri::command]
fn create_skill(app: tauri::AppHandle, name: String, description: String, system_prompt: String) -> skills::Skill {
    skills::create_skill(&app, &name, &description, &system_prompt)
}

/// 删除 Skill
#[tauri::command]
fn delete_skill(app: tauri::AppHandle, id: String) -> Result<(), String> {
    skills::delete_skill(&app, &id)
}

/// 获取单个 Skill
#[tauri::command]
fn get_skill(app: tauri::AppHandle, id: String) -> Option<skills::Skill> {
    skills::get_skill(&app, &id)
}

/// 向会话添加一条系统消息（用于应用 Skill）
#[tauri::command]
fn add_system_message(app: tauri::AppHandle, session_id: String, content: String) -> Result<(), String> {
    let mut sess = session::get_session(&app, &session_id).ok_or("会话不存在")?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sess.messages.push(session::Message {
        id: uuid::Uuid::new_v4().to_string(),
        role: session::MessageRole::System,
        content,
        created_at: now,
    });
    session::save_session(&app, &sess)
}

/// 列出目录内容（用于文件树）
#[tauri::command]
fn list_directory(app: tauri::AppHandle, path: String) -> Result<Vec<serde_json::Value>, String> {
    let settings = settings::load_settings(&app);
    let root = if !settings.working_dir.is_empty() {
        std::path::PathBuf::from(&settings.working_dir)
    } else {
        std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?
    };

    let target_dir = if path.is_empty() || path == "." {
        root.clone()
    } else {
        let p = std::path::Path::new(&path);
        if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
    };

    let mut entries = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&target_dir) {
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // 隐藏 . 开头的文件夹和 node_modules/target
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            let file_type = entry.file_type().ok();
            let is_dir = file_type.map(|t| t.is_dir()).unwrap_or(false);
            entries.push(serde_json::json!({
                "name": name,
                "path": entry.path().to_string_lossy().to_string(),
                "is_dir": is_dir,
            }));
        }
    }

    // 文件夹排前面，按名称排序
    entries.sort_by(|a, b| {
        let a_dir = a["is_dir"].as_bool().unwrap_or(false);
        let b_dir = b["is_dir"].as_bool().unwrap_or(false);
        if a_dir != b_dir { return b_dir.cmp(&a_dir); }
        a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or(""))
    });

    Ok(entries)
}

/// 启动终端
#[tauri::command]
fn terminal_start(state: tauri::State<'_, TerminalSession>, app: tauri::AppHandle) -> Result<(), String> {
    state.start(&app)
}

/// 写入终端输入
#[tauri::command]
fn terminal_write(state: tauri::State<'_, TerminalSession>, input: String) -> Result<(), String> {
    state.write(&input)
}

/// 停止终端
#[tauri::command]
fn terminal_stop(state: tauri::State<'_, TerminalSession>) -> Result<(), String> {
    state.stop()
}

/// 读取文件内容（供编辑器使用）
#[tauri::command]
fn read_file_content(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    std::fs::read_to_string(p).map_err(|e| format!("读取文件失败: {}", e))
}

/// 启动 Agent Turn（流式，通过 Tauri Event 推送）
#[tauri::command]
async fn start_agent_turn(
    app: tauri::AppHandle,
    session_id: String,
    content: String,
) -> Result<deepseek::ChatResult, String> {
    agent::r#loop::run_turn(&app, &session_id, &content).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_sessions,
            create_session,
            get_session,
            delete_session,
            rename_session,
            start_agent_turn,
            list_directory,
            read_file_content,
            terminal_start,
            terminal_write,
            terminal_stop,
            list_skills,
            create_skill,
            delete_skill,
            get_skill,
            add_system_message,
        ])
        .manage(TerminalSession::new())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            std::fs::create_dir_all(&data_dir).ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动应用时发生错误");
}
