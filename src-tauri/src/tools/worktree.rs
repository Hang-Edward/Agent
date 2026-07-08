use super::{Tool, ToolResult};
use std::process::Command;

pub struct WorktreeTool;

impl Tool for WorktreeTool {
    fn name(&self) -> &str {
        "worktree"
    }

    fn description(&self) -> &str {
        "管理 Git Worktree。参数：action（create/delete/list），branch（分支名），path（可选，worktree 路径）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["list", "create", "delete"], "description": "操作类型" },
                "branch": { "type": "string", "description": "分支名（create 时需要）" },
                "path": { "type": "string", "description": "worktree 路径" }
            },
            "required": ["action"]
        })
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("list");
        let branch = args.get("branch").and_then(|v| v.as_str()).unwrap_or("");
        let wt_path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        let dir = sandbox.allowed_dir();

        match action {
            "list" => {
                let output = Command::new("git")
                    .args(["worktree", "list"])
                    .current_dir(dir)
                    .output();
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        ToolResult { success: true, output: stdout.to_string(), tool_name: self.name().to_string() }
                    }
                    Err(e) => ToolResult { success: false, output: format!("执行失败: {}", e), tool_name: self.name().to_string() },
                }
            }
            "create" => {
                if branch.is_empty() {
                    return ToolResult { success: false, output: "缺少参数: branch".to_string(), tool_name: self.name().to_string() };
                }
                let path = if wt_path.is_empty() {
                    format!("../{}-worktree", branch)
                } else {
                    wt_path.to_string()
                };
                let output = Command::new("git")
                    .args(["worktree", "add", &path, branch])
                    .current_dir(dir)
                    .output();
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            ToolResult {
                                success: true,
                                output: format!("已创建 worktree: {} (分支: {})", path, branch),
                                tool_name: self.name().to_string(),
                            }
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            ToolResult { success: false, output: stderr.to_string(), tool_name: self.name().to_string() }
                        }
                    }
                    Err(e) => ToolResult { success: false, output: format!("执行失败: {}", e), tool_name: self.name().to_string() },
                }
            }
            "delete" => {
                let target = if wt_path.is_empty() { branch } else { wt_path };
                let output = Command::new("git")
                    .args(["worktree", "remove", target])
                    .current_dir(dir)
                    .output();
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            ToolResult { success: true, output: format!("已删除 worktree: {}", target), tool_name: self.name().to_string() }
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            ToolResult { success: false, output: stderr.to_string(), tool_name: self.name().to_string() }
                        }
                    }
                    Err(e) => ToolResult { success: false, output: format!("执行失败: {}", e), tool_name: self.name().to_string() },
                }
            }
            _ => ToolResult { success: false, output: format!("未知操作: {}", action), tool_name: self.name().to_string() },
        }
    }
}
