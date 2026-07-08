use super::{Tool, ToolResult};

pub struct GrepTool;

impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "在文件中搜索文本内容。参数：pattern（搜索模式），path（可选，限定搜索路径）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "搜索文本（支持正则）" },
                "path": { "type": "string", "description": "限定搜索路径（可选）" }
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

        // 确定搜索路径
        let search_path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => sandbox.resolve(p).unwrap_or_else(|_| sandbox.allowed_dir().to_path_buf()),
            None => sandbox.allowed_dir().to_path_buf(),
        };

        if !search_path.exists() {
            return ToolResult {
                success: false,
                output: format!("路径不存在: {}", search_path.display()),
                tool_name: self.name().to_string(),
            };
        }

        // 使用 grep 实现（通过 regex 匹配）
        let regex = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => return ToolResult {
                success: false,
                output: format!("正则表达式无效: {}", e),
                tool_name: self.name().to_string(),
            },
        };

        let mut results = Vec::new();
        let max_results = 50;
        let max_lines_per_file = 10;

        if search_path.is_dir() {
            for entry in walkdir::WalkDir::new(&search_path)
                .max_depth(10)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if results.len() >= max_results {
                    break;
                }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let matches: Vec<_> = regex
                            .find_iter(&content)
                            .take(max_lines_per_file)
                            .collect();
                        if !matches.is_empty() {
                            let path_str = entry.path().to_string_lossy();
                            for m in &matches {
                                results.push(format!("{}:{} | {}", path_str, m.start(), m.as_str()));
                            }
                        }
                    }
                }
        } else {
            if let Ok(content) = std::fs::read_to_string(&search_path) {
                for m in regex.find_iter(&content).take(max_lines_per_file) {
                    results.push(format!("{} | {}", m.start(), m.as_str()));
                }
            }
        }

        if results.is_empty() {
            ToolResult {
                success: true,
                output: format!("未找到匹配: {}", pattern),
                tool_name: self.name().to_string(),
            }
        } else {
            ToolResult {
                success: true,
                output: format!("找到 {} 处匹配:\n{}", results.len(), results.join("\n")),
                tool_name: self.name().to_string(),
            }
        }
    }
}
