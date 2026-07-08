use super::{Tool, ToolResult};
use crate::code_index;

pub struct CodeSearchTool;

impl Tool for CodeSearchTool {
    fn name(&self) -> &str {
        "code_search"
    }

    fn description(&self) -> &str {
        "搜索代码中的符号（函数、类、结构体等）。参数：query（符号名称关键词），path（可选，限定目录）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "符号名称关键词" },
                "kind": { "type": "string", "description": "可选过滤：function/class/struct/enum/trait/const" },
                "path": { "type": "string", "description": "可选限定目录" }
            },
            "required": ["query"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(q) => q.to_lowercase(),
            None => return ToolResult {
                success: false, output: "缺少参数: query".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        let kind_filter = args.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        let path_filter = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        let symbols = code_index::index_project(sandbox.allowed_dir());

        let mut results: Vec<String> = Vec::new();

        for sym in &symbols {
            if !sym.name.to_lowercase().contains(&query) {
                continue;
            }
            if !kind_filter.is_empty() && sym.kind != kind_filter {
                continue;
            }
            if !path_filter.is_empty() && !sym.file.contains(path_filter) {
                continue;
            }

            results.push(format!("{} | {}:{} | {}", sym.kind, sym.file, sym.line, sym.signature));
            if results.len() >= 30 {
                break;
            }
        }

        if results.is_empty() {
            ToolResult {
                success: true,
                output: format!("未找到包含 '{}' 的符号", query),
                tool_name: self.name().to_string(),
            }
        } else {
            ToolResult {
                success: true,
                output: format!("找到 {} 个符号:\n{}", results.len(), results.join("\n")),
                tool_name: self.name().to_string(),
            }
        }
    }
}
