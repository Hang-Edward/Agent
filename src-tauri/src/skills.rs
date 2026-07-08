use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// 单个 Skill 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    /// 注入 System Prompt 的指令模板
    pub system_prompt: String,
    pub created_at: String,
}

/// Skill 摘要（列表用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

fn skills_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("无法获取应用数据目录");
    let sk_dir = dir.join("skills");
    std::fs::create_dir_all(&sk_dir).ok();
    sk_dir
}

fn skill_path(app: &tauri::AppHandle, id: &str) -> PathBuf {
    skills_dir(app).join(format!("{}.json", id))
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn list_skills(app: &tauri::AppHandle) -> Vec<SkillSummary> {
    let dir = skills_dir(app);
    let mut skills = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(skill) = serde_json::from_str::<Skill>(&json) {
                        skills.push(SkillSummary {
                            id: skill.id,
                            name: skill.name,
                            description: skill.description,
                            created_at: skill.created_at,
                        });
                    }
                }
            }
        }
    }

    skills.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    skills
}

pub fn create_skill(app: &tauri::AppHandle, name: &str, description: &str, system_prompt: &str) -> Skill {
    let skill = Skill {
        id: new_id(),
        name: name.to_string(),
        description: description.to_string(),
        system_prompt: system_prompt.to_string(),
        created_at: now(),
    };

    let path = skill_path(app, &skill.id);
    if let Ok(json) = serde_json::to_string_pretty(&skill) {
        std::fs::write(&path, json).ok();
    }

    skill
}

pub fn delete_skill(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let path = skill_path(app, id);
    std::fs::remove_file(&path).map_err(|e| format!("删除 Skill 失败: {}", e))
}

pub fn get_skill(app: &tauri::AppHandle, id: &str) -> Option<Skill> {
    let path = skill_path(app, id);
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}
