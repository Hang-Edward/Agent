mod settings;

use settings::Settings;
use tauri::Manager;

/// Tauri 命令：获取设置
#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Settings {
    settings::load_settings(&app)
}

/// Tauri 命令：保存设置
#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    settings::save_settings(&app, &settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_settings, save_settings])
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            std::fs::create_dir_all(&data_dir).ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动应用时发生错误");
}
