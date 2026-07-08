use crate::deepseek::ChatMessage;

/// 构建系统提示词（Agent 角色定义 + 行为约束 + 工具定义）
pub fn build_system_prompt() -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    format!(
        r#"你是 Agent PC，一个桌面 AI 编码助手。
你运行在 Windows 操作系统上，帮助用户完成软件工程任务。

## 行为准则
- 你会被提供一组工具来完成文件读写、代码搜索、命令执行等操作。
- 你需要分析用户需求，规划步骤，然后使用工具逐步实现。
- 每次工具调用后，你会收到执行结果，根据结果决定下一步操作。
- 当任务完成时，或需要用户进一步指示时，输出最终回复。

## 工具使用规则
- 使用工具前先说明你要做什么。
- 如果工具调用出错，分析错误原因并重试或改用其他方法。
- 不要一次性调用过多工具，每次调 1-2 个。

## 回复风格
- 中文回复。
- 代码和文件路径保持原文。
- 使用 Markdown 格式组织回复。
- 可以使用 LaTeX 公式（$$...$$）和 Obsidian Callout（>[!note]）。

当前时间：{}

现在开始帮助用户。
"#,
        now
    )
}

/// 构建发送给 DeepSeek API 的完整消息列表
pub fn build_messages(
    system_prompt: &str,
    history: &[ChatMessage],
    user_input: &str,
) -> Vec<ChatMessage> {
    let mut messages = Vec::new();

    // 1. 系统提示
    messages.push(ChatMessage {
        role: crate::deepseek::Role::System,
        content: system_prompt.to_string(),
    });

    // 2. 历史消息
    messages.extend_from_slice(history);

    // 3. 用户当前输入
    messages.push(ChatMessage {
        role: crate::deepseek::Role::User,
        content: user_input.to_string(),
    });

    messages
}
