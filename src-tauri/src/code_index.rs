use std::path::Path;

/// 代码索引条目
#[derive(Debug, Clone)]
pub struct SymbolIndex {
    pub name: String,
    pub kind: String,      // function, class, struct, interface, const
    pub file: String,
    pub line: usize,
    pub signature: String,
}

/// 使用正则构建简单的代码符号索引（支持 Rust/TS/JS/Python 等）
pub fn index_project(project_dir: &Path) -> Vec<SymbolIndex> {
    let mut symbols = Vec::new();

    // 定义各语言的符号提取规则
    let patterns: Vec<(&str, &str)> = vec![
        // Rust
        (r"(?m)^\s*(pub\s+)?fn\s+(\w+)", "function"),
        (r"(?m)^\s*(pub\s+)?struct\s+(\w+)", "struct"),
        (r"(?m)^\s*(pub\s+)?enum\s+(\w+)", "enum"),
        (r"(?m)^\s*(pub\s+)?trait\s+(\w+)", "trait"),
        (r"(?m)^\s*(pub\s+)?impl\s+(\w+)", "impl"),
        (r"(?m)^\s*(pub\s+)?(const|static)\s+(\w+)", "const"),
        // TypeScript/JavaScript
        (r"(?m)^\s*(export\s+)?(function|class|interface|type|enum)\s+(\w+)", "decl"),
        (r"(?m)^\s*(export\s+)?(const|let|var)\s+(\w+)\s*[:=]\s*(\(|async|function)", "function"),
    ];

    // 支持的文件类型
    let extensions = ["rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "cpp", "h"];

    // 遍历项目目录
    for entry in walkdir::WalkDir::new(project_dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !extensions.contains(&ext) {
                continue;
            }
            // 跳过 node_modules/target
            let pstr = path.to_string_lossy();
            if pstr.contains("node_modules") || pstr.contains("target") || pstr.contains(".git") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let file = pstr.to_string();
                for (pattern, kind) in &patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        for cap in re.captures_iter(&content) {
                            // 提取名称（在最后一个捕获组）
                            let name = cap
                                .iter()
                                .last()
                                .flatten()
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_default();
                            if name.is_empty() || name == "pub" {
                                continue;
                            }
                            // 计算行号
                            let line = content[..cap.get(0).unwrap().start()]
                                .matches('\n')
                                .count()
                                + 1;

                            symbols.push(SymbolIndex {
                                name,
                                kind: kind.to_string(),
                                file: file.clone(),
                                line,
                                signature: cap.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }

    // 限制最大索引条目，按文件路径排序
    symbols.truncate(2000);
    symbols.sort_by(|a, b| a.file.cmp(&b.file));
    symbols
}
