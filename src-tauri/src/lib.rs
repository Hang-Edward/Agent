mod settings;
mod session;

use settings::Settings;
use tauri::Manager;

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
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            std::fs::create_dir_all(&data_dir).ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动应用时发生错误");
}
