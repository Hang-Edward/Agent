use crate::agent::types::*;
use crate::deepseek;
use crate::session::{Message, MessageRole};
use tauri::Emitter;
use uuid::Uuid;

/// 运行 Agent Turn Loop：
/// 1. 构建上下文（系统提示 + 历史 + 用户输入）
/// 2. 流式调用 DeepSeek API
/// 3. 通过 Tauri Event 推送 token/reasoning 到前端
/// 4. 将 AI 回复保存到会话
/// 5. 返回结果
pub async fn run_turn(
    app: &tauri::AppHandle,
    session_id: &str,
    user_input: &str,
) -> Result<deepseek::ChatResult, String> {
    // 1. 加载设置
    let settings = crate::settings::load_settings(app);
    let system_prompt = crate::agent::context::build_system_prompt();

    // 2. 获取会话历史
    let mut sess = crate::session::get_session(app, session_id).ok_or("会话不存在")?;

    // 3. 保存用户消息
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let user_msg = Message {
        id: Uuid::new_v4().to_string(),
        role: MessageRole::User,
        content: user_input.to_string(),
        created_at: now.clone(),
    };
    sess.messages.push(user_msg);

    // 4. 构建上下文
    let history: Vec<deepseek::ChatMessage> = sess
        .messages
        .iter()
        .map(|m| {
            let role = match m.role {
                MessageRole::User => deepseek::Role::User,
                MessageRole::Assistant => deepseek::Role::Assistant,
                MessageRole::System => deepseek::Role::System,
            };
            deepseek::ChatMessage {
                role,
                content: m.content.clone(),
            }
        })
        .collect();

    let messages = crate::agent::context::build_messages(&system_prompt, &history, user_input);

    // 5. 流式调用 API
    let app_token = app.clone();
    let app_reasoning = app.clone();
    let result = deepseek::chat_completion_stream(
        &settings.api_key,
        &settings.model,
        &messages,
        // on_token
        move |token: &str| {
            let _ = app_token.emit(EVENT_TOKEN, TokenPayload {
                token: token.to_string(),
            });
        },
        // on_reasoning
        move |reasoning: &str| {
            let _ = app_reasoning.emit(EVENT_REASONING, ReasoningPayload {
                reasoning: reasoning.to_string(),
            });
        },
        // on_done
        |_input_tokens: u32, _output_tokens: u32| {},
    )
    .await?;

    // 6. 保存 AI 回复
    let ai_msg = Message {
        id: Uuid::new_v4().to_string(),
        role: MessageRole::Assistant,
        content: result.content.clone(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    sess.messages.push(ai_msg);
    crate::session::save_session(app, &sess)?;

    // 7. 如果会话名称为默认，用用户消息前 20 字命名
    if sess.name.starts_with("新对话") {
        let preview: String = user_input.chars().take(20).collect();
        let name = if user_input.len() > 20 {
            format!("{}...", preview)
        } else {
            preview
        };
        crate::session::rename_session(app, session_id, &name).ok();
    }

    // 8. 发送完成事件
    let _ = app.emit(
        EVENT_DONE,
        DonePayload {
            content: result.content.clone(),
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
        },
    );

    Ok(result)
}
