use serde::{Deserialize, Serialize};

/* ───── API 请求/响应类型 ───── */

/// 消息角色
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// 对话消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

/// 请求体（发送给 DeepSeek API）
#[derive(Debug, Serialize)]
struct RequestBody {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

/// API 返回的顶层结构
#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// 聊天补全的最终结果
#[derive(Debug, Serialize)]
pub struct ChatResult {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/* ───── 主接口 ───── */

const API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

/// 发送聊天请求到 DeepSeek API（非流式）
pub async fn chat_completion(
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<ChatResult, String> {
    if api_key.is_empty() {
        return Err("请在设置中配置 API Key".to_string());
    }

    let body = RequestBody {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: false,
    };

    let client = reqwest::Client::new();

    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let status = resp.status();

    // 解析错误响应
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("API 错误 ({}): {}", status.as_u16(), body_text));
    }

    let api_resp: ApiResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let content = api_resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    let (input_tokens, output_tokens) = match api_resp.usage {
        Some(u) => (u.prompt_tokens, u.completion_tokens),
        None => (0, 0),
    };

    Ok(ChatResult {
        content,
        input_tokens,
        output_tokens,
    })
}

/* ───── 流式 SSE 相关 ───── */

/// 流式 delta 事件
#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<Usage>,
}

/// 流式回调：处理 DeepSeek 的 SSE 事件流
pub async fn chat_completion_stream<F1, F2, F3>(
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
    on_token: F1,
    on_reasoning: F2,
    on_done: F3,
) -> Result<ChatResult, String>
where
    F1: Fn(&str) + Send + Sync + 'static,
    F2: Fn(&str) + Send + Sync + 'static,
    F3: Fn(u32, u32) + Send + Sync + 'static,
{
    if api_key.is_empty() {
        return Err("请在设置中配置 API Key".to_string());
    }

    let body = RequestBody {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: true,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("API 错误 ({}): {}", status.as_u16(), body_text));
    }

    let mut full_content = String::new();
    let mut input_tokens: u32 = 0;
    let mut output_tokens: u32 = 0;

    // 使用 buf 按行读取 SSE 流
    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("读取流失败: {}", e))?;
        let text = String::from_utf8_lossy(&chunk);
        buf.push_str(&text);

        // 按行处理（SSE 每行以 \n 结尾）
        loop {
            let line_end = match buf.find('\n') {
                Some(pos) => pos,
                None => break, // 等待更多数据
            };

            let line = buf[..line_end].trim().to_string();
            buf = buf[line_end + 1..].to_string();

            if line.is_empty() {
                continue; // SSE 空行 = 分隔符
            }

            if !line.starts_with("data: ") {
                continue;
            }

            let data = &line[6..]; // 去掉 "data: " 前缀

            if data == "[DONE]" {
                break;
            }

            // 解析 JSON
            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                for choice in chunk.choices {
                    if let Some(content) = &choice.delta.content {
                        if !content.is_empty() {
                            on_token(content);
                            full_content.push_str(content);
                        }
                    }
                    if let Some(reasoning) = &choice.delta.reasoning_content {
                        if !reasoning.is_empty() {
                            on_reasoning(reasoning);
                        }
                    }
                }

                if let Some(usage) = &chunk.usage {
                    input_tokens = usage.prompt_tokens;
                    output_tokens = usage.completion_tokens;
                }
            }
        }
    }

    on_done(input_tokens, output_tokens);

    Ok(ChatResult {
        content: full_content,
        input_tokens,
        output_tokens,
    })
}
