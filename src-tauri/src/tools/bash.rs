use super::ToolResult;
use crate::tools::bash_safety::{self, RiskLevel};
use std::process::Command;

pub struct BashTool;

impl super::Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "在 Windows 终端中执行命令。支持 PowerShell 和 cmd 命令。参数：command（要执行的命令），timeout（可选，超时秒数，默认 30）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的命令（PowerShell 语法）"
                },
                "timeout": {
                    "type": "integer",
                    "description": "超时秒数，默认 30",
                    "default": 30
                }
            },
            "required": ["command"]
        })
    }

    fn is_read_only(&self) -> bool {
        // 某些命令是只读的（如 ls/dir/git status），但这里统一返回 false
        // 具体安全验证由 bash_safety 模块处理
        false
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let command = match args.get("command").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult {
                success: false,
                output: "缺少参数: command".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        // 安全验证
        let verdict = bash_safety::validate(command);
        match verdict.level {
            RiskLevel::Blocked => {
                return ToolResult {
                    success: false,
                    output: format!("[安全拦截] {}", verdict.reason),
                    tool_name: self.name().to_string(),
                };
            }
            RiskLevel::Caution => {
                // 目前先允许，后续可加入审批流程
                // 对于 caution 级别的命令，在结果中附加警告
            }
            RiskLevel::Safe => {}
        }

        let timeout_secs = args.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);

        // 执行命令（Windows 下使用 PowerShell）
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .current_dir(sandbox.allowed_dir())
            .output();

        match output {
            Ok(out) => {
                let mut result = String::new();

                if verdict.level == RiskLevel::Caution {
                    result.push_str(&format!("[警告] {}\n\n", verdict.reason));
                }

                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if !stdout.is_empty() {
                    result.push_str(&format!("{}\n", stdout));
                }
                if !stderr.is_empty() {
                    result.push_str(&format!("[stderr]\n{}", stderr));
                }

                if out.status.success() {
                    result.push_str(&format!("\n[命令完成，退出码: {}]", out.status.code().unwrap_or(-1)));
                } else {
                    result.push_str(&format!("\n[命令失败，退出码: {}]", out.status.code().unwrap_or(-1)));
                }

                ToolResult {
                    success: out.status.success(),
                    output: result,
                    tool_name: self.name().to_string(),
                }
            }
            Err(e) => ToolResult {
                success: false,
                output: format!("执行命令失败: {}", e),
                tool_name: self.name().to_string(),
            },
        }
    }
}
