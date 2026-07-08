/// Bash 命令安全验证器
/// 阻止危险命令，防止 Agent 执行破坏性操作

/// 命令风险等级
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,       // 可自动执行
    Caution,    // 需用户确认
    Blocked,    // 直接拦截
}

/// 验证结果
#[derive(Debug)]
pub struct SafetyVerdict {
    pub level: RiskLevel,
    pub reason: String,
}

/// 危险命令列表（直接拦截）
const BLOCKED_COMMANDS: &[&str] = &[
    "rm", "rmdir", "del", "erase", "rd",        // 删除
    "format", "fdisk", "diskpart",               // 磁盘
    "shutdown", "reboot", "restart", "halt",     // 系统
    "init", "poweroff",
    "dd", "mkfs", "mkswap",                      // 底层磁盘操作
    "reg delete", "reg add",                     // 注册表
    "cipher", "sfc", "bcdedit",                  // 系统关键工具
];

/// 需要谨慎的命令
const CAUTION_COMMANDS: &[&str] = &[
    "chmod", "chown", "attrib",                   // 权限修改
    "taskkill", "kill", "pkill",                  // 进程终止
    "net user", "net localgroup",                 // 用户管理
    "wmic", "powershell remove-item",
    "takeown", "icacls",                          // 所有权/权限
    "move", "ren", "rename",                      // 移动/重命名
    "curl", "wget", "iwr",                        // 网络下载
    "reg query", "regedit",                       // 注册表查看
    "sc", "net start", "net stop",                // 服务管理
];

/// 验证命令安全性
pub fn validate(command: &str) -> SafetyVerdict {
    let cmd_lower = command.trim().to_lowercase();

    // 检查空白命令
    if cmd_lower.is_empty() {
        return SafetyVerdict {
            level: RiskLevel::Safe,
            reason: String::new(),
        };
    }

    // 检查是否被拦截
    for &blocked in BLOCKED_COMMANDS {
        if cmd_lower.starts_with(blocked) || cmd_lower.contains(&format!(" {} ", blocked)) {
            return SafetyVerdict {
                level: RiskLevel::Blocked,
                reason: format!("命令 '{}' 被拦截：{}", command, blocked_desc(blocked)),
            };
        }
    }

    // 检查是否需要谨慎
    for &caution in CAUTION_COMMANDS {
        if cmd_lower.starts_with(caution) || cmd_lower.contains(&format!(" {} ", caution)) {
            return SafetyVerdict {
                level: RiskLevel::Caution,
                reason: format!("命令 '{}' 包含 {}，请注意影响", command, caution),
            };
        }
    }

    // 检查危险模式
    if cmd_lower.contains("rm -rf") || cmd_lower.contains("rm -r") {
        return SafetyVerdict {
            level: RiskLevel::Caution,
            reason: "递归删除操作，请确认".to_string(),
        };
    }

    if cmd_lower.contains(">") && cmd_lower.contains("format") {
        return SafetyVerdict {
            level: RiskLevel::Blocked,
            reason: "发现格式化命令重定向".to_string(),
        };
    }

    SafetyVerdict {
        level: RiskLevel::Safe,
        reason: String::new(),
    }
}

fn blocked_desc(cmd: &str) -> &str {
    match cmd {
        "rm" | "rmdir" | "del" => "删除操作被拦截",
        "format" | "fdisk" => "磁盘操作被拦截",
        "shutdown" | "reboot" => "系统操作被拦截",
        "dd" => "底层磁盘写入被拦截",
        _ => "危险命令被拦截",
    }
}
