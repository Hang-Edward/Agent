use regex::Regex;

/// 解析出的工具调用
#[derive(Debug)]
pub struct ParsedToolCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// 从 AI 回复文本中提取所有工具调用
pub fn parse_tool_calls(response: &str) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();

    // 匹配 <tool_call>...</tool_call> 块
    let block_re = Regex::new(r"<tool_call>(.*?)</tool_call>").unwrap();
    let name_re = Regex::new(r"<tool_name>(.*?)</tool_name>").unwrap();
    let args_re = Regex::new(r"<tool_args>(.*?)</tool_args>").unwrap();

    for cap in block_re.captures_iter(response) {
        let block = cap.get(1).unwrap().as_str();

        let name = name_re
            .captures(block)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string());

        let args = args_re
            .captures(block)
            .and_then(|c| c.get(1))
            .and_then(|m| serde_json::from_str(m.as_str().trim()).ok());

        match (name, args) {
            (Some(name), Some(args)) => {
                calls.push(ParsedToolCall { name, args });
            }
            _ => continue,
        }
    }

    calls
}
