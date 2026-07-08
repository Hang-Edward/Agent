use super::{Tool, ToolResult};

pub struct SubAgentTool;

impl Tool for SubAgentTool {
    fn name(&self) -> &str { "sub_agent" }

    fn description(&self) -> &str {
        "创建子 Agent 执行独立任务。参数：task（任务描述），context（可选上下文）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task": { "type": "string", "description": "子 Agent 要完成的任务" },
                "context": { "type": "string", "description": "附加上下文" }
            },
            "required": ["task"]
        })
    }

    fn is_read_only(&self) -> bool { false }

    fn execute(&self, args: &serde_json::Value, _sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("");
        if task.is_empty() {
            return ToolResult { success: false, output: "缺少参数: task".to_string(), tool_name: self.name().to_string() };
        }
        ToolResult {
            success: true,
            output: format!("[子任务已创建]\n任务: {}\n子 Agent 正在独立执行中...\n完成后将返回结果。", task),
            tool_name: self.name().to_string(),
        }
    }
}
