use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    /// 环境变量（可选）
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// 单个 MCP 工具描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: Value,
}

/// MCP 客户端：管理一个 MCP 服务器进程
pub struct McpClient {
    config: McpServerConfig,
    process: Mutex<Option<Child>>,
    tools: Mutex<Vec<McpTool>>,
}

impl McpClient {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            process: Mutex::new(None),
            tools: Mutex::new(Vec::new()),
        }
    }

    /// 连接并初始化 MCP 服务器
    pub fn connect(&self) -> Result<(), String> {
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动 MCP 服务器 '{}' 失败: {}", self.config.name, e))?;

        // 发送 initialize 请求
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "0.1.0",
                "capabilities": {},
                "clientInfo": { "name": "agent-pc", "version": "0.1.0" }
            },
            "id": 1
        });

        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "{}", serde_json::to_string(&init_req).unwrap())
                .map_err(|e| format!("写入 MCP 初始化请求失败: {}", e))?;
        }

        // 读取响应
        let response = if let Some(ref mut stdout) = child.stdout {
            let mut reader = BufReader::new(stdout);
            let mut resp = String::new();
            reader.read_line(&mut resp).map_err(|e| format!("读取 MCP 响应失败: {}", e))?;
            resp
        } else {
            return Err("MCP 服务器没有 stdout".to_string());
        };

        // 发送 tools/list 请求
        let list_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 2
        });

        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "{}", serde_json::to_string(&list_req).unwrap())
                .map_err(|e| format!("写入 MCP tools/list 失败: {}", e))?;
        }

        // 读取 tools/list 响应
        if let Some(ref mut stdout) = child.stdout {
            let mut reader = BufReader::new(stdout);
            let mut resp = String::new();
            reader.read_line(&mut resp).map_err(|e| format!("读取 MCP tools/list 失败: {}", e))?;

            if let Ok(parsed) = serde_json::from_str::<Value>(&resp) {
                if let Some(tools) = parsed.get("result").and_then(|r| r.get("tools")) {
                    let tool_list: Vec<McpTool> = serde_json::from_value(tools.clone())
                        .unwrap_or_default();
                    let mut store = self.tools.lock().map_err(|e| format!("锁定失败: {}", e))?;
                    *store = tool_list;
                }
            }
        }

        *self.process.lock().map_err(|e| format!("锁定失败: {}", e))? = Some(child);
        Ok(())
    }

    /// 获取已发现的工具列表
    pub fn get_tools(&self) -> Vec<McpTool> {
        self.tools.lock().ok().map(|t| t.clone()).unwrap_or_default()
    }

    /// 调用 MCP 工具
    pub fn call_tool(&self, name: &str, args: &Value) -> Result<String, String> {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": { "name": name, "arguments": args },
            "id": 3
        });

        let mut proc = self.process.lock().map_err(|e| format!("锁定失败: {}", e))?;
        let child = proc.as_mut().ok_or("MCP 服务器未连接")?;

        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "{}", serde_json::to_string(&req).unwrap())
                .map_err(|e| format!("写入 MCP 调用失败: {}", e))?;
        }

        if let Some(ref mut stdout) = child.stdout {
            let mut reader = BufReader::new(stdout);
            let mut resp = String::new();
            reader.read_line(&mut resp).map_err(|e| format!("读取 MCP 响应失败: {}", e))?;

            if let Ok(parsed) = serde_json::from_str::<Value>(&resp) {
                if let Some(error) = parsed.get("error") {
                    return Err(format!("MCP 错误: {}", error));
                }
                if let Some(result) = parsed.get("result") {
                    if let Some(content) = result.get("content") {
                        // content is an array of content items
                        if let Some(text) = content.as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|c| c.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            return Ok(text.to_string());
                        }
                        return Ok(serde_json::to_string_pretty(content).unwrap_or_default());
                    }
                    return Ok(serde_json::to_string_pretty(result).unwrap_or_default());
                }
            }
        }

        Err("MCP 调用无响应".to_string())
    }

    /// 断开连接
    pub fn disconnect(&self) {
        if let Ok(mut proc) = self.process.lock() {
            if let Some(mut child) = proc.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
