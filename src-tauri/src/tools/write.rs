use super::{Tool, ToolResult};

pub struct WriteTool;

impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "写入文件（新建或覆盖）。参数：path（路径），content（内容）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "文件路径" },
                "content": { "type": "string", "description": "文件内容" }
            },
            "required": ["path", "content"]
        })
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult {
                success: false, output: "缺少参数: path".to_string(),
                tool_name: self.name().to_string(),
            },
        };
        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult {
                success: false, output: "缺少参数: content".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        let resolved = match sandbox.resolve(path) {
            Ok(p) => p,
            Err(e) => return ToolResult {
                success: false, output: e,
                tool_name: self.name().to_string(),
            },
        };

        // 确保父目录存在
        if let Some(parent) = resolved.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        match std::fs::write(&resolved, content) {
            Ok(_) => ToolResult {
                success: true,
                output: format!("已写入 {} ({} 字节)", resolved.display(), content.len()),
                tool_name: self.name().to_string(),
            },
            Err(e) => ToolResult {
                success: false,
                output: format!("写入失败: {}", e),
                tool_name: self.name().to_string(),
            },
        }
    }
}
