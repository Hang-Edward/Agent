use std::path::{Path, PathBuf};

/// 文件沙箱：确保所有文件操作均在允许的工作目录内
pub struct Sandbox {
    allowed_dir: PathBuf,
}

impl Sandbox {
    /// 创建沙箱，指定允许访问的根目录
    pub fn new(allowed_dir: PathBuf) -> Self {
        // 规范化路径
        let canonical = std::fs::canonicalize(&allowed_dir).unwrap_or(allowed_dir);
        Self { allowed_dir: canonical }
    }

    /// 解析并验证路径：返回规范化的绝对路径，如果越界则返回错误
    pub fn resolve(&self, path: &str) -> Result<PathBuf, String> {
        let p = Path::new(path);
        let resolved = if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.allowed_dir.join(p)
        };

        // 规范化（移除 ../ 等）
        let canonical = std::fs::canonicalize(&resolved)
            .map_err(|e| format!("路径无效: {} ({})", path, e))?;

        // 检查是否在允许目录内
        if !canonical.starts_with(&self.allowed_dir) {
            return Err(format!(
                "路径越界: {} (不允许访问沙箱外的文件)",
                canonical.display()
            ));
        }

        Ok(canonical)
    }

    /// 获取允许的根目录
    pub fn allowed_dir(&self) -> &Path {
        &self.allowed_dir
    }
}
