use crate::agent::tool_parser::{parse_tool_calls, ParsedToolCall};
use crate::agent::types::*;
use crate::deepseek::{self, ChatMessage};
use crate::sandbox::Sandbox;
use crate::session::{self, Message, MessageRole};
use crate::settings;
use crate::tools::ToolRegistry;
use tauri::Emitter;
use uuid::Uuid;

const MAX_ITERATIONS: usize = 15;

/// 运行 Agent Turn（含工具调用循环）
pub async fn run_turn(
    app: &tauri::AppHandle,
    session_id: &str,
    user_input: &str,
) -> Result<deepseek::ChatResult, String> {
    let handle = app.clone();
    let settings = settings::load_settings(app);
    let registry = ToolRegistry::default();
    let system_prompt = crate::agent::context::build_system_prompt(&registry);

    // 加载会话
    let mut sess = session::get_session(app, session_id).ok_or("会话不存在")?;

    // 保存用户消息
    let now = || chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sess.messages.push(Message {
        id: Uuid::new_v4().to_string(),
        role: MessageRole::User,
        content: user_input.to_string(),
        created_at: now(),
    });

    // 工作目录沙箱：优先使用设置中的工作目录，否则用当前项目目录
    let sandbox_dir = if !settings.working_dir.is_empty() {
        std::path::PathBuf::from(&settings.working_dir)
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    };
    let sandbox = Sandbox::new(sandbox_dir);

    // 累加最终输出
    let mut full_output = String::new();
    let mut total_input_tokens: u32 = 0;
    let mut total_output_tokens: u32 = 0;

    for iteration in 0..MAX_ITERATIONS {
        // 构建历史消息
        let history: Vec<ChatMessage> = sess
            .messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => deepseek::Role::User,
                    MessageRole::Assistant => deepseek::Role::Assistant,
                    MessageRole::System => deepseek::Role::System,
                };
                ChatMessage {
                    role,
                    content: m.content.clone(),
                }
            })
            .collect();

        // 本轮输入：第一次用用户输入，后续用 "请继续"
        let current_input = if iteration == 0 {
            user_input.to_string()
        } else {
            "请继续，根据之前的工具执行结果推进任务。".to_string()
        };

        let messages =
            crate::agent::context::build_messages(&system_prompt, &history, &current_input);

        // 流式调用 API
        let h1 = handle.clone();
        let h2 = handle.clone();
        let result = deepseek::chat_completion_stream(
            &settings.api_key,
            &settings.model,
            &messages,
            move |token: &str| {
                let _ = h1.emit(EVENT_TOKEN, TokenPayload {
                    token: token.to_string(),
                });
            },
            move |reasoning: &str| {
                let _ = h2.emit(EVENT_REASONING, ReasoningPayload {
                    reasoning: reasoning.to_string(),
                });
            },
            |_in: u32, _out: u32| {},
        )
        .await?;

        let iteration_content = result.content.clone();
        full_output.push_str(&iteration_content);
        total_input_tokens += result.input_tokens;
        total_output_tokens += result.output_tokens;

        // 检查是否有工具调用
        let tool_calls = parse_tool_calls(&iteration_content);

        if tool_calls.is_empty() {
            // 没有工具调用 → AI 最终回复，保存并退出
            sess.messages.push(Message {
                id: Uuid::new_v4().to_string(),
                role: MessageRole::Assistant,
                content: iteration_content,
                created_at: now(),
            });
            session::save_session(app, &sess)?;
            break;
        }

        // 有工具调用：先保存 AI 回复（含工具调用标记）
        sess.messages.push(Message {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: iteration_content,
            created_at: now(),
        });

        // 逐个执行工具
        for tc in &tool_calls {
            let result = execute_tool(&registry, tc, &sandbox);

            // 发送工具结果事件
            let _ = handle.emit(
                EVENT_TOOL_RESULT,
                ToolResultPayload {
                    id: Uuid::new_v4().to_string(),
                    name: tc.name.clone(),
                    result: result.output.clone(),
                },
            );

            // 将工具结果作为系统消息加入上下文
            sess.messages.push(Message {
                id: Uuid::new_v4().to_string(),
                role: MessageRole::System,
                content: format!(
                    "[工具执行: {}]\n{}",
                    tc.name,
                    if result.success { result.output } else { format!("错误: {}", result.output) }
                ),
                created_at: now(),
            });
        }

        session::save_session(app, &sess)?;

        // 继续下一轮循环，让 AI 处理工具结果
    }

    // 更新会话名称
    if sess.name.starts_with("新对话") {
        let preview: String = user_input.chars().take(20).collect();
        let name = if user_input.len() > 20 {
            format!("{}...", preview)
        } else {
            preview
        };
        session::rename_session(app, session_id, &name).ok();
    }

    // 发送完成事件
    let _ = handle.emit(
        EVENT_DONE,
        DonePayload {
            content: full_output.clone(),
            input_tokens: total_input_tokens,
            output_tokens: total_output_tokens,
        },
    );

    Ok(deepseek::ChatResult {
        content: full_output,
        input_tokens: total_input_tokens,
        output_tokens: total_output_tokens,
    })
}

/// 执行单个工具
fn execute_tool(
    registry: &ToolRegistry,
    call: &ParsedToolCall,
    sandbox: &Sandbox,
) -> crate::tools::ToolResult {
    match registry.find(&call.name) {
        Some(tool) => tool.execute(&call.args, sandbox),
        None => crate::tools::ToolResult {
            success: false,
            output: format!("未知工具: {}", call.name),
            tool_name: call.name.clone(),
        },
    }
}
