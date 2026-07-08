use super::{Tool, ToolResult};

pub struct WebSearchTool;
pub struct WebFetchTool;

impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "搜索互联网。参数：query（搜索关键词），max_results（可选，返回结果数，默认5）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "搜索关键词" },
                "max_results": { "type": "integer", "description": "返回结果数量，默认5", "default": 5 }
            },
            "required": ["query"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn execute(&self, args: &serde_json::Value, _sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return ToolResult {
                success: false, output: "缺少参数: query".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        // 使用 DuckDuckGo Instant Answer API（免费，无需 Key）
        let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding(query));

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return ToolResult {
                success: false, output: format!("创建运行时失败: {}", e),
                tool_name: self.name().to_string(),
            },
        };
        let result = rt.block_on(async {
            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build() {
                Ok(c) => c,
                Err(e) => return Err(format!("创建客户端失败: {}", e)),
            };

            let resp = match client.get(&url)
                .header("User-Agent", "Agent-PC/1.0")
                .send().await {
                Ok(r) => r,
                Err(e) => return Err(format!("搜索请求失败: {}", e)),
            };

            match resp.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("读取响应失败: {}", e)),
            }
        });

        match result {
            Ok(body) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                    let mut output = String::new();

                    // Abstract
                    if let Some(abstract_text) = parsed.get("AbstractText").and_then(|t| t.as_str()) {
                        if !abstract_text.is_empty() {
                            output.push_str(&format!("摘要: {}\n\n", abstract_text));
                        }
                    }

                    // Related topics (搜索结果)
                    if let Some(results) = parsed.get("RelatedTopics").and_then(|t| t.as_array()) {
                        for (i, r) in results.iter().enumerate() {
                            if i >= args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5) as usize {
                                break;
                            }
                            if let (Some(text), Some(url)) = (
                                r.get("Text").and_then(|t| t.as_str()),
                                r.get("FirstURL").and_then(|u| u.as_str())
                            ) {
                                output.push_str(&format!("{}. {} - {}\n", i+1, text, url));
                            }
                        }
                    }

                    if output.is_empty() {
                        output = format!("未找到 '{}' 的相关结果", query);
                    }

                    ToolResult {
                        success: true,
                        output,
                        tool_name: self.name().to_string(),
                    }
                } else {
                    ToolResult {
                        success: true,
                        output: format!("搜索结果(原始): {}", &body[..body.len().min(2000)]),
                        tool_name: self.name().to_string(),
                    }
                }
            }
            Err(e) => ToolResult {
                success: false, output: e,
                tool_name: self.name().to_string(),
            },
        }
    }
}

impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "获取网页内容。参数：url（网页链接）"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "网页 URL" }
            },
            "required": ["url"]
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn execute(&self, args: &serde_json::Value, _sandbox: &crate::sandbox::Sandbox) -> ToolResult {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return ToolResult {
                success: false, output: "缺少参数: url".to_string(),
                tool_name: self.name().to_string(),
            },
        };

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return ToolResult {
                success: false, output: format!("创建运行时失败: {}", e),
                tool_name: self.name().to_string(),
            },
        };
        let result = rt.block_on(async {
            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build() {
                Ok(c) => c,
                Err(e) => return Err(format!("创建客户端失败: {}", e)),
            };

            let resp = match client.get(url)
                .header("User-Agent", "Agent-PC/1.0")
                .send().await {
                Ok(r) => r,
                Err(e) => return Err(format!("请求失败: {}", e)),
            };

            let status = resp.status();
            let body = match resp.text().await {
                Ok(b) => b,
                Err(e) => return Err(format!("读取失败: {}", e)),
            };

            // 简单提取文本（去除 HTML 标签）
            let text = strip_html_tags(&body);
            let preview: String = text.chars().take(5000).collect();
            let truncated = if text.len() > 5000 { "\n...(截断)" } else { "" };

            Ok(format!("[状态码: {}]\n{}{}", status.as_u16(), preview, truncated))
        });

        match result {
            Ok(output) => ToolResult { success: true, output, tool_name: self.name().to_string() },
            Err(e) => ToolResult { success: false, output: e, tool_name: self.name().to_string() },
        }
    }
}

/// 简单去除 HTML 标签，提取文本
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let html_lower = html.to_lowercase();

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !in_tag {
            if chars[i] == '<' {
                in_tag = true;
                // 检查是否为 script/style 标签
                if html_lower[i..].starts_with("<script") {
                    in_script = true;
                } else if html_lower[i..].starts_with("<style") {
                    in_style = true;
                }
            } else {
                result.push(chars[i]);
            }
            i += 1;
        } else {
            if chars[i] == '>' {
                in_tag = false;
                if in_script && html_lower[i.saturating_sub(8)..=i].contains("/script>") {
                    in_script = false;
                }
                if in_style && html_lower[i.saturating_sub(6)..=i].contains("/style>") {
                    in_style = false;
                }
            } else if in_script && html_lower[i..].starts_with("</script>") {
                in_script = false;
            } else if in_style && html_lower[i..].starts_with("</style>") {
                in_style = false;
            }
            i += 1;
        }
    }

    // 清理多余空白
    let cleaned: String = result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    cleaned
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        other => format!("%{:02X}", other as u8),
    }).collect()
}
