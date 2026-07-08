pub mod bash;
pub mod bash_safety;
pub mod edit;
pub mod glob;
pub mod grep;
pub mod read;
pub mod write;

use serde::Serialize;

/// 工具执行结果
#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub tool_name: String,
}

/// 工具 trait：所有工具需实现此接口
pub trait Tool: Send + Sync {
    /// 工具名称（供模型调用）
    fn name(&self) -> &str;

    /// 工具描述（嵌入 System Prompt 供模型理解用途）
    fn description(&self) -> &str;

    /// 工具参数 JSON Schema（供模型生成参数）
    fn parameters(&self) -> serde_json::Value;

    /// 是否只读（只读工具自动执行，无需审批）
    fn is_read_only(&self) -> bool;

    /// 执行工具
    fn execute(&self, args: &serde_json::Value, sandbox: &crate::sandbox::Sandbox) -> ToolResult;
}

/// 工具注册中心
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// 注册一个工具
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// 获取所有工具
    pub fn all(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    /// 按名称查找工具
    pub fn find(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    /// 生成所有工具的 JSON Schema 描述（嵌入 System Prompt）
    pub fn tools_schema(&self) -> serde_json::Value {
        let arr: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "parameters": t.parameters(),
                })
            })
            .collect();
        serde_json::Value::Array(arr)
    }

    /// 创建默认工具集
    pub fn default() -> Self {
        let mut reg = Self::new();
        reg.register(Box::new(read::ReadTool));
        reg.register(Box::new(write::WriteTool));
        reg.register(Box::new(edit::EditTool));
        reg.register(Box::new(glob::GlobTool));
        reg.register(Box::new(grep::GrepTool));
        reg.register(Box::new(bash::BashTool));
        reg
    }
}
