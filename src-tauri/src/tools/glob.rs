use super::{Tool, ToolResult};

pub struct GlobTool;

impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "按通配符模式搜索文件名。参数：pattern（glob 模式，如 **/*.rs）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "glob 模式，如 **/*.rs, src/**/*.ts"
                }
            },
            "required": ["pattern"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult {
                success: false, output: "缺少参数: pattern".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        // 使用 glob 包
        let full_pattern = sandbox.allowed_dir().join(pattern);

        match glob::glob(&full_pattern.to_string_lossy()) {
            Ok(paths) => {
                let matches: Vec<String> = paths
                    .flatten()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                if matches.is_empty() {
                    ToolResult {
                        success: true,
                        output: format!("无匹配文件: {}", pattern),
                        tool_name: self.name().to_string(),
                    }
                } else {
                    ToolResult {
                        success: true,
                        output: format!("找到 {} 个匹配:\n{}", matches.len(), matches.join("\n")),
                        tool_name: self.name().to_string(),
                    }
                }
            }
            Err(e) => ToolResult {
                success: false,
                output: format!("搜索失败: {}", e),
                tool_name: self.name().to_string(),
            },
        }
    }
}
