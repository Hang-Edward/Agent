use super::{Tool, ToolResult};

pub struct EditTool;

impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "精确替换文件中的文本。参数：path（路径），old（原文本），new（新文本）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "文件路径" },
                "old": { "type": "string", "description": "待替换的原文（必须完全匹配）" },
                "new": { "type": "string", "description": "替换后的文本" }
            },
            "required": ["path", "old", "new"]
        })
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let old = args.get("old").and_then(|v| v.as_str()).unwrap_or("");
        let new = args.get("new").and_then(|v| v.as_str()).unwrap_or("");

        if path.is_empty() || old.is_empty() {
            return ToolResult {
                success: false, output: "缺少参数: path, old".to_string(),
                tool_name: self.name().to_string(),
            };
        }

        let resolved = match sandbox.resolve(path) {
            Ok(p) => p,
            Err(e) => return ToolResult {
                success: false, output: e,
                tool_name: self.name().to_string(),
            },
        };

        let content = match std::fs::read_to_string(&resolved) {
            Ok(c) => c,
            Err(e) => return ToolResult {
                success: false,
                output: format!("读取文件失败: {}", e),
                tool_name: self.name().to_string(),
            },
        };

        if !content.contains(old) {
            return ToolResult {
                success: false,
                output: format!("未找到匹配的文本:\n---\n{}\n---\n请提供完整的上下文，确保精确匹配。", old),
                tool_name: self.name().to_string(),
            };
        }

        let new_content = content.replace(old, new);
        match std::fs::write(&resolved, &new_content) {
            Ok(_) => {
                let diff_count = content.matches(old).count();
                ToolResult {
                    success: true,
                    output: format!("已替换 {} 处文本，文件: {}", diff_count, resolved.display()),
                    tool_name: self.name().to_string(),
                }
            }
            Err(e) => ToolResult {
                success: false,
                output: format!("写入失败: {}", e),
                tool_name: self.name().to_string(),
            },
        }
    }
}
