# PC 桌面端 AI 编码 Agent — 完整规划方案

> 基于 Codex/Claude Code 架构调研，目标：开发一款原生桌面 Agent，接入 DeepSeek API。

---

## 一、整体架构

```
┌─────────────────────────────────────────────────────────┐
│                   Desktop UI (Tauri 2)                    │
│  ┌────────────────────────────────────────────────────┐  │
│  │            Frontend: React 19 + TypeScript + Vite   │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────┐  │  │
│  │  │ 对话面板  │ │ 文件树   │ │ Diff视图  │ │终端  │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────┘  │  │
│  └──────────────────────┬─────────────────────────────┘  │
│                         │ Tauri IPC (invoke + events)    │
│  ┌──────────────────────▼─────────────────────────────┐  │
│  │          Rust 后端 (Agent Runtime Engine)           │  │
│  │                                                    │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐  │  │
│  │  │ Turn Loop│ │ 工具系统 │ │ 权限/安全层       │  │  │
│  │  └──────────┘ └──────────┘ └───────────────────┘  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐  │  │
│  │  │ 上下文管理│ │ 会话持久 │ │ MCP扩展           │  │  │
│  │  └──────────┘ └──────────┘ └───────────────────┘  │  │
│  └──────────────────────┬─────────────────────────────┘  │
└─────────────────────────┼───────────────────────────────┘
                          │ HTTPS + SSE
              ┌───────────▼────────────┐
              │   DeepSeek API         │
              │  (deepseek-v4-pro/flash)│
              └────────────────────────┘
```

### 核心设计原则

- **Less scaffolding, more model** — Agent 循环信任模型判断，不写死路由/分类器
- **高性能 Rust 核心** — Turn Loop、权限、沙箱用 Rust 实现，前端只管渲染
- **富 UI 体验** — 不是终端工具，而是完整的桌面应用（原生窗口、分栏、实时流式渲染）
- **先做核心功能，再逐步扩展** — MVP = 对话 + 文件编辑 + 命令执行 + 最小权限模型

---

## 二、技术栈选型

| 层级 | 技术 | 选型理由 |
|------|------|----------|
| **桌面框架** | **Tauri 2** | 二进制 < 15MB，原生性能，Windows 原生支持完善，Rust 后端 |
| **前端框架** | **React 19 + TypeScript + Vite** | 生态成熟，组件化 UI，HMR 开发体验好 |
| **样式** | **Tailwind CSS v4 + shadcn/ui** | 快速构建美观 UI，组件库丰富 |
| **状态管理** | **Zustand** | 轻量高效，适合 AI 聊天类应用 |
| **终端** | **xterm.js** (前端) + **portable-pty** (Rust) | 完整终端模拟，支持 Windows |
| **Diff 视图** | **react-diff-viewer-continued** 或 monaco-editor diff | 代码变更对比 |
| **编辑器** | **Monaco Editor** (VS Code 内核) | 完整代码编辑体验 |
| **AI SDK** | 自研 Rust Agent Loop (直接调用 DeepSeek API) | 完全控制，无框架依赖 |
| **序列化** | **serde** + **Zod** (前端) | 类型安全的数据校验 |
| **持久化** | **SQLite** (tauri-plugin-sql) | 会话历史、设置存储 |
| **Markdown 渲染** | **react-markdown** + **rehype-highlight** | AI 回复渲染 |

---

## 三、DeepSeek API 接入细节

### API 兼容性

DeepSeek 提供 **OpenAI 兼容 API**，可使用标准 OpenAI SDK 或原生 HTTP 调用：

```
POST https://api.deepseek.com/v1/chat/completions
Authorization: Bearer <api-key>
```

### 关键参数

| 参数 | 值 | 说明 |
|------|-----|------|
| `model` | `deepseek-v4-pro` / `deepseek-v4-flash` | Pro 更强但更贵，Flash 更快更便宜 |
| `stream` | `true` | 流式 SSE 输出，实现打字机效果 |
| `tools` | 自定义工具列表 | 文件读写、Bash、搜索等 |
| `max_tokens` | 4096~8192 | 按需设置 |
| `temperature` | 0.0~0.7 | 编码场景建议 0.1~0.3 |

### 流式处理流程

```
用户输入 → 构造请求 → SSE 连接 →
  while (收到响应):
    if token: 渲染到对话气泡
    if reasoning_content: 显示思考过程
    if tool_calls: 解析 → 执行 → 结果追加 → 继续请求
  done → 返回最终文本
```

### 关键注意点

- **DeepSeek 使用 XML 风格工具调用**：`<｜｜DSML｜｜tool_calls>...`，需要在流式解析时处理跨 chunk 边界
- **reasoning_content** 字段包含思考过程，可展示给用户提升透明度
- **前缀缓存 (Prefix Caching)**：将静态内容放在 prompt 前方，可大幅降低成本（实测最高 99.82% 缓存命中率）

---

## 四、Agent Runtime 引擎设计

### 4.1 Turn Loop（核心循环）

```
loop {
    // 1. 构造请求（系统 prompt + 历史 + 新工具结果 + 用户输入）
    let request = build_request(context);

    // 2. 流式请求 DeepSeek API
    let stream = deepseek_client.chat_stream(request);

    // 3. 解析 SSE 事件流
    for event in stream {
        match event {
            Token(text) => emit_to_ui(text),
            Reasoning(text) => emit_to_ui_reasoning(text),
            ToolCall(name, args) => {
                // 4. 执行工具
                let result = execute_tool(name, args).await;

                // 5. 结果追加到上下文，继续循环
                context.add_tool_result(name, result);
                break; // 跳出当前流，发送新请求
            },
            Done => return final_text,
        }
    }
}
```

### 4.2 工具系统

每个工具的定义：

```rust
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> JsonSchema;  // JSON Schema 供模型理解
    fn is_read_only(&self) -> bool;
    fn execute(&self, args: JsonValue) -> Result<String>;
}
```

#### MVP 核心工具

| 工具 | 功能 | 只读 | 需要确认 |
|------|------|------|---------|
| `read` | 读取文件内容 | ✅ | ❌ |
| `write` | 写入/覆盖文件 | ❌ | ✅ |
| `edit` | 精确替换文件内容 | ❌ | ✅ |
| `glob` | 搜索文件名 | ✅ | ❌ |
| `grep` | 搜索文件内容 | ✅ | ❌ |
| `bash` | 执行 Shell 命令 | 视情况 | 视情况 |
| `search_web` | 网络搜索 | ✅ | ❌ |
| `finish` | 结束任务 | - | - |

### 4.3 上下文管理

```
对话结构:
  System Prompt (静态，缓存优化优先)
  ├── 角色定义
  ├── 工具定义 (JSON Schema)
  ├── 安全规则
  └── 工作目录信息
  ──── ← SYSTEM_PROMPT_DYNAMIC_BOUNDARY
  User Context (动态)
  ├── CLAUDE.md / AGENTS.md 项目规则
  ├── 当前工作目录状态
  └── 用户偏好

  历史消息 (滑动窗口 / 压缩)
  ├── 最近的 N 轮对话
  ├── 关键工具结果摘要
  └── 压缩后的历史总结
```

**窗口管理策略**：
- 保持最近的 30~50 条消息完整
- 超出的部分自动压缩为结构化摘要
- 压缩失败时有熔断机制（最多重试 3 次）

---

## 五、UI 设计

### 5.1 主界面布局

```
┌──────────────────────────────────────────────────────┐
│  Title Bar: [Agent Name] - [Project]     [⚙] [□] [×]│
├──────────┬───────────────────┬───────────────────────┤
│          │                   │                       │
│  侧边栏   │    对话主面板      │   右侧面板            │
│  (可折叠) │                   │                       │
│          │  ┌─────────────┐  │  ┌─────────────────┐  │
│  📁 文件树│  │ 用户消息气泡  │  │  Diff 预览        │  │
│          │  └─────────────┘  │  │                  │  │
│  📋 会话  │  ┌─────────────┐  │  │  +1 -2 行       │  │
│  列表     │  │ AI 回复     │  │  │  ...            │  │
│          │  │ (流式渲染)   │  │  │                  │  │
│  🔧 工具  │  │ [思考过程]   │  │  └─────────────────┘  │
│  历史     │  │ [工具调用]   │  │                       │
│          │  │ [执行结果]   │  │  ┌─────────────────┐  │
│          │  └─────────────┘  │  │  终端面板          │  │
│          │                   │  │  (xterm.js)       │  │
│          │  ┌─────────────┐  │  │                  │  │
│          │  │ 输入框 + 按钮 │  │  │  $ git status    │  │
│          │  └─────────────┘  │  │  ...              │  │
│          │                   │  │  └─────────────────┘  │
├──────────┴───────────────────┴───────────────────────┤
│  Status Bar: [Model: deepseek-v4-flash] [Tokens: 1.2k]│
└──────────────────────────────────────────────────────┘
```

### 5.2 核心交互流程

1. **对话模式** — 在输入框输入自然语言指令，AI 回复 + 自动执行工具
2. **流式渲染** — AI 回复实时显示，思考过程折叠可展开，工具调用实时展示进度
3. **Diff 确认** — 文件修改时右侧显示 diff，用户可以接受/拒绝
4. **命令执行** — 终端面板实时显示命令输出，内置中断（Ctrl+C）
5. **审批流程** — 高风险操作（写文件、执行命令）弹出确认对话框

### 5.3 辅助功能

- **会话管理** — 多会话 Tab、会话历史、会话重命名
- **设置面板** — API Key 管理、模型选择、工具权限配置
- **Token 统计** — 实时显示 token 消耗和预估费用
- **进度追踪** — 多步骤任务时显示子任务进度条
- **快捷键** — Ctrl+Enter 发送、Ctrl+L 清屏、Ctrl+K 等

---

## 六、分阶段实施计划

### Phase 1: 基础骨架（2-3 周）

- [x] Tauri 2 项目初始化 + React 19 + TypeScript + Tailwind
- [x] 主窗口框架：三栏布局（侧边栏/对话/右侧面板）
- [x] 对话 UI 组件：气泡、输入框、流式文本渲染
- [x] DeepSeek API 基础接入（非流式对话）
- [x] 设置页面：API Key 配置、模型选择
- [x] 基础 Session 管理（新建/切换/删除）

### Phase 2: Agent 核心（3-4 周）

- [x] Rust Agent Loop 实现（Turn Loop）
- [x] 流式 SSE 接入 + 实时渲染
- [x] 思考过程（reasoning_content）解析与显示
- [x] 工具系统框架：Tool trait + 注册机制
- [x] 核心工具：read / write / edit / glob / grep
- [x] 文件系统沙箱（限制操作范围到项目目录）
- [x] 审批确认对话框（高风险操作）
- [x] Diff 视图（文件修改前后对比）

### Phase 3: 增强功能（3-4 周）

- [x] Bash 工具 + 终端面板（xterm.js + portable-pty）
- [x] Bash 安全验证（命令黑名单/白名单）
- [x] 上下文窗口管理（压缩/截断策略）
- [x] Monaco Editor 集成（文件编辑）
- [x] 文件树侧边栏
- [x] 多会话 Tab 支持
- [x] 会话历史持久化（SQLite）
- [x] MCP 协议支持（扩展工具生态）

### Phase 4: 生产化（2-3 周）

- [x] 深色/浅色主题切换
- [x] 快捷键系统
- [x] 设置面板完善（代理配置、权限预设等）
- [x] 日志/调试面板
- [x] 打包分发（msi/nsis 安装包）
- [x] 自动更新
- [x] 性能优化（大文件、长对话）

### Phase 5: 进阶特性（持续迭代）

- [x] 多 Agent 并行（Git Worktree 隔离）
- [x] 子 Agent 系统（复杂任务分解）
- [x] Skills 系统（可复用的指令模板）
- [x] AGENTS.md 项目级规则支持
- [x] Web 搜索工具
- [x] 代码索引 + 语义搜索
- [x] 自定义 MCP 服务器
- [x] 插件系统

---

## 七、关键文件结构

```
agent-pc/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs              # 入口，Tauri setup
│   │   ├── agent/
│   │   │   ├── mod.rs
│   │   │   ├── loop.rs          # Turn Loop 引擎
│   │   │   ├── context.rs       # 上下文管理
│   │   │   └── stream.rs        # SSE 流解析
│   │   ├── tools/
│   │   │   ├── mod.rs
│   │   │   ├── registry.rs      # 工具注册中心
│   │   │   ├── read.rs
│   │   │   ├── write.rs
│   │   │   ├── edit.rs
│   │   │   ├── bash.rs          # + 安全验证器
│   │   │   ├── glob.rs
│   │   │   └── grep.rs
│   │   ├── sandbox/
│   │   │   ├── mod.rs
│   │   │   └── fs.rs            # 文件系统沙箱
│   │   ├── session/
│   │   │   ├── mod.rs
│   │   │   └── store.rs         # SQLite 会话存储
│   │   ├── mcp/
│   │   │   ├── mod.rs
│   │   │   └── client.rs        # MCP 客户端
│   │   └── commands.rs          # Tauri IPC 命令
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # React 前端
│   ├── App.tsx                  # 根组件
│   ├── main.tsx                 # 入口
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── MainPanel.tsx
│   │   │   ├── RightPanel.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── chat/
│   │   │   ├── ChatView.tsx
│   │   │   ├── MessageBubble.tsx
│   │   │   ├── StreamingText.tsx
│   │   │   ├── ThinkingBlock.tsx
│   │   │   └── ToolCallCard.tsx
│   │   ├── editor/
│   │   │   ├── FileTree.tsx
│   │   │   ├── MonacoEditor.tsx
│   │   │   └── DiffView.tsx
│   │   ├── terminal/
│   │   │   └── TerminalPanel.tsx
│   │   └── settings/
│   │       ├── SettingsDialog.tsx
│   │       └── ApiKeyInput.tsx
│   ├── stores/
│   │   ├── chatStore.ts
│   │   ├── sessionStore.ts
│   │   ├── settingsStore.ts
│   │   └── fileStore.ts
│   ├── hooks/
│   │   ├── useAgentStream.ts
│   │   └── useDeepSeek.ts
│   └── lib/
│       ├── api.ts               # DeepSeek API 客户端
│       └── types.ts             # 类型定义
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tailwind.config.ts
```

---

## 八、参考项目

### 核心参考
| 项目 | 借鉴点 |
|------|--------|
| **Claude Code** | Turn Loop 设计、工具系统、权限模型、上下文压缩策略 |
| **OpenAI Codex** | App Server 架构（JSON-RPC）、Thread/Turn/Item 模型、多 Agent 编排 |
| **DeepSeek TUI** (Rust) | 子 Agent 系统、LSP 集成、前缀缓存优化 |
| **Zagens** (Tauri + Rust) | 长任务支持、Completion Gate、Checkpoint UI |
| **Devo** (Rust) | 多模型支持、MCP 集成、权限工具执行 |

### UI 参考
| 项目 | 借鉴点 |
|------|--------|
| **Cursor** | 三栏布局、Diff 确认流程 |
| **Windsurf** | 对话 + 代码编辑一体化 |
| **VSCode** | 文件树、终端面板、Monaco Editor |

---

## 九、关键挑战与对策

| 挑战 | 对策 |
|------|------|
| **DeepSeek 工具调用格式特殊** | 实现 DSML 解析器，处理跨 chunk 边界的 XML 标签拼接 |
| **SSE 流式解析健壮性** | 状态机解析器 + 异常恢复 + 重试机制 |
| **大文件编辑** | 分块读取 + 差异比较（diff），避免全文件传输 |
| **Bash 安全** | 命令解析 AST → 验证器链（黑名单/白名单/敏感参数），全部在 Rust 侧完成 |
| **上下文超限** | 滑动窗口 + LLM 压缩 + 熔断重试上限 |
| **Windows 兼容性** | Tauri 原生支持 Windows，Bash 工具适配 PowerShell + cmd + WSL |
| **启动速度** | Rust 编译优化 + 懒加载模块 + 并行初始化 |

---

## 十、MVP 验收标准

- [ ] 用户可以在桌面应用中输入自然语言指令
- [ ] AI 响应实时流式渲染，思考过程可见
- [ ] 可以读取、编辑、创建项目文件
- [ ] 可以执行 Shell 命令并在终端面板查看输出
- [ ] 文件修改时显示 Diff 对比，用户可以确认/拒绝
- [ ] 高风险操作有审批弹窗
- [ ] API Key 和模型可在设置中配置
- [ ] 会话可保存、恢复
- [ ] 打包为 Windows 安装程序

---

> **下一步：** 如果你同意这个规划方向，我们可以直接进入 Phase 1 的开发——从初始化 Tauri 2 项目和搭建基础 UI 骨架开始。
