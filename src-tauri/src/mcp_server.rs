use crate::sandbox::Sandbox;
use crate::tools::ToolRegistry;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// 简单的 MCP 服务器：通过 stdio 提供 JSON-RPC 接口
/// 其他 AI 应用可以通过此服务器调用 Agent PC 的工具
pub struct McpServer {
    registry: ToolRegistry,
    sandbox: Sandbox,
}

impl McpServer {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            registry: ToolRegistry::default(),
            sandbox: Sandbox::new(working_dir),
        }
    }

    /// 启动 MCP 服务器（stdio 模式）
    pub fn start(&self) {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line);
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            let _ = writeln!(handle, "{}", response);
            let _ = handle.flush();
        }
    }

    fn handle_request(&self, request: &str) -> String {
        let parsed: Value = match serde_json::from_str(request) {
            Ok(v) => v,
            Err(e) => return self.error(-32700, format!("JSON 解析错误: {}", e)),
        };

        let id = parsed.get("id");
        let method = parsed.get("method").and_then(|m| m.as_str()).unwrap_or("");

        match method {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => {
                let params = parsed.get("params");
                self.handle_tools_call(id, params)
            }
            _ => self.error_with_id(id, -32601, format!("未知方法: {}", method)),
        }
    }

    fn handle_initialize(&self, id: Option<&Value>) -> String {
        self.response(id, serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "agent-pc",
                "version": "0.1.0"
            }
        }))
    }

    fn handle_tools_list(&self, id: Option<&Value>) -> String {
        let tools: Vec<Value> = self.registry.all().iter().map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
                "inputSchema": t.parameters(),
            })
        }).collect();

        self.response(id, serde_json::json!({ "tools": tools }))
    }

    fn handle_tools_call(&self, id: Option<&Value>, params: Option<&Value>) -> String {
        let name = params
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let args = params
            .and_then(|p| p.get("arguments"))
            .unwrap_or(&serde_json::Value::Null);

        let tool = match self.registry.find(name) {
            Some(t) => t,
            None => return self.error_with_id(id, -32602, format!("未知工具: {}", name)),
        };

        let result = tool.execute(args, &self.sandbox);

        self.response(id, serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": result.output,
                }
            ],
            "isError": !result.success,
        }))
    }

    fn response(&self, id: Option<&Value>, result: Value) -> String {
        let mut resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
        });
        if let Some(id) = id {
            resp["id"] = id.clone();
        }
        serde_json::to_string(&resp).unwrap_or_default()
    }

    fn error(&self, code: i32, message: String) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": code, "message": message },
        }).to_string()
    }

    fn error_with_id(&self, id: Option<&Value>, code: i32, message: String) -> String {
        let mut resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": code, "message": message },
        });
        if let Some(id) = id {
            resp["id"] = id.clone();
        }
        serde_json::to_string(&resp).unwrap_or_default()
    }
}
