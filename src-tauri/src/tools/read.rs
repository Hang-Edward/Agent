use super::{Tool, ToolResult};

pub struct ReadTool;

impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "读取文件内容。参数：path（文件路径，相对或绝对）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件路径"
                }
            },
            "required": ["path"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult {
                success: false,
                output: "缺少参数: path".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        let resolved = match sandbox.resolve(path) {
            Ok(p) => p,
            Err(e) => return ToolResult {
                success: false,
                output: e,
                tool_name: self.name().to_string(),
            },
        };

        if !resolved.exists() {
            return ToolResult {
                success: false,
                output: format!("文件不存在: {}", resolved.display()),
                tool_name: self.name().to_string(),
            };
        }

        if resolved.is_dir() {
            // 读取目录：列出文件
            match std::fs::read_dir(&resolved) {
                Ok(entries) => {
                    let files: Vec<String> = entries
                        .flatten()
                        .map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            let meta = e.metadata().ok();
                            let is_dir = meta.map(|m| m.is_dir()).unwrap_or(false);
                            if is_dir { format!("{}/", name) } else { name }
                        })
                        .collect();
                    ToolResult {
                        success: true,
                        output: files.join("\n"),
                        tool_name: self.name().to_string(),
                    }
                }
                Err(e) => ToolResult {
                    success: false,
                    output: format!("读取目录失败: {}", e),
                    tool_name: self.name().to_string(),
                },
            }
        } else {
            match std::fs::read_to_string(&resolved) {
                Ok(content) => {
                    let lines = content.lines().count();
                    let preview: String = content.chars().take(5000).collect();
                    let truncated = if content.len() > 5000 { "\n...(截断)" } else { "" };
                    ToolResult {
                        success: true,
                        output: format!("{} ({} 行):\n{}{}", resolved.display(), lines, preview, truncated),
                        tool_name: self.name().to_string(),
                    }
                }
                Err(e) => ToolResult {
                    success: false,
                    output: format!("读取文件失败: {}", e),
                    tool_name: self.name().to_string(),
                },
            }
        }
    }
}
