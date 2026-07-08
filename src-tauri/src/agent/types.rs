use serde::Serialize;

/// Agent 发送给前端的流式事件
#[derive(Debug, Clone, Serialize)]
pub struct TokenPayload {
    pub token: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningPayload {
    pub reasoning: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallPayload {
    pub name: String,
    pub args: serde_json::Value,
    pub id: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ToolResultPayload {
    pub id: String,
    pub name: String,
    pub result: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct DonePayload {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub error: String,
}

/// 事件名称常量
pub const EVENT_TOKEN: &str = "agent:token";
pub const EVENT_REASONING: &str = "agent:reasoning";
pub const EVENT_TOOL_CALL: &str = "agent:tool_call";
pub const EVENT_TOOL_RESULT: &str = "agent:tool_result";
pub const EVENT_DONE: &str = "agent:done";
pub const EVENT_ERROR: &str = "agent:error";
