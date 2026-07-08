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
