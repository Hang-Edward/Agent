use crate::deepseek::ChatMessage;
use crate::tools::ToolRegistry;
use std::path::Path;

/// 查找项目根目录下的 AGENTS.md 文件
fn load_agents_md(project_dir: &Path) -> String {
    for name in &["AGENTS.md", "CLAUDE.md", ".cursorrules"] {
        let path = project_dir.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let preview: String = content.chars().take(2000).collect();
                return format!("\n## 项目规则 ({}):\n{}\n", name, preview);
            }
        }
    }
    String::new()
}

/// 构建系统提示词
pub fn build_system_prompt(registry: &ToolRegistry, project_dir: &Path) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let tools_schema = registry.tools_schema();
    let agents_rules = load_agents_md(project_dir);

    format!(
        r#"你是 Agent PC，一个桌面 AI 编码助手。

## 行为准则
- 分析需求，规划步骤，使用工具逐步实现。
- 每个工具调用后根据结果决定下一步。
- 任务完成或需要用户指示时输出最终回复。

## 可用工具
{}

## 工具调用格式
当需要使用工具时，在回复中按以下格式输出：

<tool_call>
<tool_name>工具名称</tool_name>
<tool_args>
{{"参数1": "值1", "参数2": "值2"}}
</tool_args>
</tool_call>

工具执行后结果会自动加入上下文，你可以继续决策。

## 回复风格
- 中文回复，Markdown 格式。
- 可以使用 LaTeX $$...$$ 和 Obsidian Callout >[!note]。
{}
当前时间：{}
"#,
        serde_json::to_string_pretty(&tools_schema).unwrap_or_default(),
        agents_rules,
        now,
    )
}

/// 构建发送给 DeepSeek API 的完整消息列表
pub fn build_messages(
    system_prompt: &str,
    history: &[ChatMessage],
    user_input: &str,
) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    messages.push(ChatMessage {
        role: crate::deepseek::Role::System,
        content: system_prompt.to_string(),
    });
    messages.extend_from_slice(history);
    messages.push(ChatMessage {
        role: crate::deepseek::Role::User,
        content: user_input.to_string(),
    });
    messages
}
