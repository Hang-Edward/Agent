# Desktop Agent PC — 完整实现流程手册

> 桌面 AI 编码 Agent，Tauri 2 + React 19 + Rust + DeepSeek API
> 主题：One Dark Pro
> 仓库：https://github.com/Hang-Edward/Agent.git

---

## 〇、环境准备（首次搭建步骤）

### 0.1 所需工具

| 工具 | 版本 | 安装位置 |
|------|------|---------|
| Rust / Cargo | 1.96.1 | `C:\Users\<user>\.cargo\` |
| VS 2022 Build Tools (MSVC) | 14.44 | `D:\应用-Applications\VS2022-BuildTools` |
| Node.js | 24.14.1 | 系统默认 |
| npm | 11.11.0 | 随 Node.js |

### 0.2 安装步骤

```powershell
# 1. 安装 Rust (rustup)
# 从 https://rustup.rs 下载 rustup-init.exe，或通过命令行：
curl -o "$env:TEMP\rustup-init.exe" "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y --default-toolchain stable

# 2. 安装 VS 2022 Build Tools（C++ 工作负载）
# 下载 vs_BuildTools.exe 并安装到 D:\应用-Applications\VS2022-BuildTools
vs_BuildTools.exe --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --quiet --wait --norestart --installPath "D:\应用-Applications\VS2022-BuildTools"

# 3. 将 cargo 加入 PATH
$env:PATH = "C:\Users\$env:USERNAME\.cargo\bin;$env:PATH"
```

### 0.3 验证安装

```powershell
rustc --version          # → rustc 1.96.1
cargo --version          # → cargo 1.96.1
node --version           # → v24.14.1
npm --version            # → 11.11.0
# MSVC 编译器位置:
D:\应用-Applications\VS2022-BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\cl.exe
```

---

## 一、项目初始化（Phase 1.1）

### 1.1 目录和 Git

```powershell
# 在 D:\VScode Projects\Agent 中操作
git init
git remote add origin https://github.com/Hang-Edward/Agent.git
```

### 1.2 创建文件结构

```
D:\VScode Projects\Agent/
├── .gitignore            # Rust/Node/IDE 忽略规则
├── CLAUDE.md             # 项目级 AI 指令
├── DESKTOP_AGENT_PLAN.md # 本文档
├── package.json          # Node 项目配置
├── index.html            # Vite 入口 HTML
├── vite.config.ts        # Vite 配置（端口 1420）
├── tsconfig.json         # TS 根配置
├── tsconfig.app.json     # 应用 TS 配置
├── tsconfig.node.json    # Node 侧 TS 配置
├── src/                  # React 前端源码
│   ├── main.tsx          # 入口
│   ├── App.tsx           # 根组件
│   ├── App.css           # 全局样式（One Dark Pro）
│   └── vite-env.d.ts     # Vite 类型声明
├── src-tauri/            # Rust 后端
│   ├── Cargo.toml        # Rust 依赖
│   ├── build.rs          # Tauri 构建脚本
│   ├── tauri.conf.json   # Tauri 配置（窗口 1280x800）
│   ├── capabilities/     # Tauri 2 权限
│   │   └── default.json
│   ├── icons/            # 应用图标
│   └── src/
│       ├── main.rs       # Windows 入口
│       └── lib.rs        # Tauri App 配置 + 命令注册
├── public/               # 静态资源
└── memory-agent-pc.md    # 项目内存记录
```

### 1.3 关键配置

**package.json 核心依赖**：
- `@tauri-apps/api` `@tauri-apps/cli` `@tauri-apps/plugin-shell` `@tauri-apps/plugin-dialog` `@tauri-apps/plugin-fs`
- `react` `react-dom` — UI 框架
- `zustand` — 状态管理
- `react-markdown` `remark-math` `rehype-katex` `rehype-highlight` `remark-gfm` `katex` — Markdown 渲染
- `highlight.js` — 代码高亮

**Cargo.toml 核心依赖**：
- `tauri` `tauri-plugin-shell` `tauri-plugin-dialog` `tauri-plugin-fs` — Tauri 框架
- `serde` `serde_json` — 序列化
- `keyring` — Windows 凭据管理器
- `uuid` `chrono` — ID 生成和日期
- `reqwest` — HTTP 客户端（DeepSeek API）
- `tokio` — 异步运行时

**tauri.conf.json 窗口配置**：
- 窗口尺寸：1280×800，最小 900×600
- 前端端口：1420
- CSP：null（允许全部）

### 1.4 初始提交

```powershell
git add -A
git commit -m "Initial project scaffold: Tauri 2 + React 19 + DeepSeek-ready"
git push -u origin master
```

---

## 二、设置面板（Phase 1.2）

### 2.1 Rust 后端：设置模块

**文件**：`src-tauri/src/settings.rs`

**功能**：
- `Settings` 结构体：`api_key`, `model`, `permission_level`, `working_dir`
- `PermissionLevel` 枚举：`Default` / `Review` / `Full`
- API Key 通过 `keyring` crate 存入 Windows 凭据管理器
- 其他设置存入 `%APPDATA%/com.agent-pc.app/settings.json`
- 启动时自动创建数据目录

**Tauri 命令**：
```rust
#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Settings        // 读取设置
fn save_settings(app: tauri::AppHandle, settings: Settings) // 保存设置
```

### 2.2 前端：设置对话框

**Store**：`src/stores/settingsStore.ts`（Zustand）
- `settings` + `dialogOpen` + `loaded` 三个状态
- `load()` / `save()` 通过 Tauri IPC 调用 Rust

**组件**：`src/components/SettingsDialog.tsx`
- 模态对话框，半透明遮罩
- API Key 输入（密码框 + 显示/隐藏切换）
- 模型选择下拉框（Flash / Pro）
- 权限级别三选一（默认 / 自动审查 / 完全访问）
- 工作目录输入

**组件**：`src/components/StatusBar.tsx`
- 底部显示：模型 | 权限级别 | API 状态
- 齿轮按钮 → 打开设置对话框

### 2.3 设置存储结构

```
%APPDATA%/com.agent-pc.app/
├── settings.json       # 设置（不含 API Key）
└── sessions/           # 会话目录
    ├── {uuid}.json     # 每个会话一个文件
    └── ...
```

Windows 凭据管理器（通过 keyring）：
- 服务名：`agent-pc`
- 用户名：`deepseek-api-key`
- 值：API Key 原文

### 2.4 提交记录

```
commit: Implement settings panel with encrypted API key storage
- Rust backend: Settings struct, keyring integration, JSON file storage
- Zustand store + SettingsDialog modal + StatusBar
- Permission levels: default / review / full
- Model: flash / pro
```

---

## 三、三栏布局 + 仪表盘（Phase 1.3）

### 3.1 UI 状态管理

**Store**：`src/stores/uiStore.ts`
```typescript
leftPanelOpen: boolean
rightPanelOpen: boolean
toggleLeftPanel() / toggleRightPanel()
```

### 3.2 左栏：会话列表

**组件**：`src/components/SessionList.tsx`
- 折叠/展开按钮（▶ / ◀）
- 「新对话」按钮
- 会话列表（点击切换，高亮当前）
- 鼠标悬浮显示重命名(✎) / 删除(✕) 按钮
- 底栏显示会话数量

### 3.3 中栏：对话区

**组件**：`src/components/ChatArea.tsx`
- 消息列表（按角色区分样式）
- 空状态 → 显示欢迎语
- 加载中 → 显示 loading
- 输入框 + 发送按钮（Enter 提交）

### 3.4 右栏：仪表盘

**组件**：`src/components/Dashboard.tsx`

五个小部件：
1. **任务步骤** — 步骤清单模式
   - 示例：分析需求 ✓ / 读取文件 ✓ / 生成方案 ◌ ⟳ / 执行修改 ○ / 验证 ○
   - 已完成(✓) 划掉，进行中(◌) 旋转动画，待办(○)，失败(✕)
2. **Token 消耗** — 输入/输出/总计/费用
3. **缓存命中率** — 占位（DeepSeek 暂未暴露）
4. **Git 状态** — 占位（分支/变更文件）
5. **Diff 概览** — 占位（+新增/-删除）

### 3.5 提交记录

```
commit: Implement three-column collapsible layout with dashboard
- Collapsible left/right panels with transition animation
- Session list + Chat area + Dashboard panels
- Step checklist, token usage, cache/git/diff widgets
```

---

## 四、会话管理（Phase 1.4）

### 4.1 Rust 后端：会话模块

**文件**：`src-tauri/src/session.rs`

**数据结构**：
```rust
Session { id, name, created_at, updated_at, messages: Vec<Message> }
Message { id, role: User|Assistant|System, content, created_at }
SessionSummary { id, name, created_at, updated_at, message_count }
```

**存储**：JSON 文件，每个会话 `{app_data}/sessions/{id}.json`

**CRUD 命令**：
```rust
list_sessions()    → Vec<SessionSummary>    // 按更新时间降序
create_session()   → Session                 // 自动生成 ID 和名称
get_session(id)    → Option<Session>         // 含完整消息列表
delete_session(id) → Result<(), String>      // 删除文件
rename_session(id, name) → Result<(), String>
save_session(app, &Session) → Result<(), String>  // 完整覆盖写入
```

### 4.2 前端：会话 Store

**Store**：`src/stores/sessionStore.ts`
- `sessions[]` / `currentId` / `currentSession` / `loading`
- `loadSessions()` / `createSession()` / `switchSession(id)` / `deleteSession(id)` / `renameSession(id, name)`

### 4.3 前端：会话列表交互

- 新建 → 自动切换到新会话
- 点击 → 切换并加载消息
- 悬浮显示 ✎ ✕
- 重命名 → 行内 input，Enter/Blur 保存
- 删除 → confirm 确认

### 4.4 提交记录

```
commit: Implement session management with CRUD operations
- JSON file storage per session
- Frontend store with all CRUD actions
- Inline rename, delete with confirm
- User/assistant bubble styles
```

---

## 五、DeepSeek API 接入（Phase 1.5）

### 5.1 Rust 后端：API 客户端

**文件**：`src-tauri/src/deepseek.rs`

**请求**：
```rust
POST https://api.deepseek.com/v1/chat/completions
Authorization: Bearer {api_key}
{
  "model": "deepseek-v4-flash",
  "messages": [{ "role": "user", "content": "..." }],
  "stream": false
}
```

**响应**：
```rust
ChatResult { content: String, input_tokens: u32, output_tokens: u32 }
```

**错误处理**：
- API Key 为空 → 返回"请在设置中配置 API Key"
- 网络失败 → 返回具体错误信息
- HTTP 错误 → 返回状态码 + 响应体

### 5.2 Tauri 命令：send_message

**命令**：`send_message(app, session_id, content)`

**完整流程**：
1. 加载设置（API Key + Model）
2. 获取当前会话
3. 保存用户消息到会话
4. 将会话历史转换为 DeepSeek 消息格式
5. 调用 DeepSeek API
6. 保存 AI 回复到会话
7. 如果会话名称为默认（"新对话..."），取第一条消息前 20 字作为名称
8. 返回 ChatResult

### 5.3 前端：发送流程

**Store 扩展**：`sessionStore.sendMessage(content)`
1. 本地插入用户消息（即时显示）
2. 设置 `sending = true`
3. 调用 `invoke("send_message", { sessionId, content })`
4. 重新加载完整会话
5. 更新 Token 统计（按 DeepSeek V4 定价计算费用）
6. 设置 `sending = false`

**Token 价格**：
| 模型 | 输入 ($/M tokens) | 输出 ($/M tokens) |
|------|-------------------|-------------------|
| deepseek-v4-flash | 0.15 | 0.6 |
| deepseek-v4-pro | 2.0 | 8.0 |

### 5.4 提交记录

```
commit: Connect DeepSeek API with chat flow end-to-end
- Rust HTTP client for chat completions
- send_message command with full session integration
- Token stats in dashboard (input/output/cost)
- "thinking..." animation during API call
```

---

## 六、Markdown 渲染（Phase 1.6）

### 6.1 MarkdownBlock 组件

**文件**：`src/components/MarkdownBlock.tsx`

**渲染管线**：
```
Markdown 文本
  → react-markdown
    → remarkObsidianCallout (自定义插件)
    → remarkGfm (表格/删除线)
    → remarkMath (LaTeX 解析)
    → rehypeKatex (LaTeX → HTML)
    → rehypeHighlight (代码高亮)
  → 自定义 components (Callout/代码/表格)
  → React 元素
```

### 6.2 Obsidian Callout 支持

**插件**：`remarkObsidianCallout()`
- 遍历 AST 的 `blockquote` 节点
- 检测首行 `>[!type]` 模式
- 标记节点属性 `data-callout` + `data-title`

**组件**：`src/components/ObsidianCallout.tsx`

**支持类型**：note, info, tip, warning, danger, success, question, bug, example, quote

**样式**：每个类型独立颜色（One Dark Pro 语义色）、左侧 4px 色条、标题背景色

**用法**：
```markdown
>[!warning] 注意
> 这里是警告内容，会渲染为带图标的彩色 callout 块。
```

### 6.3 样式覆盖

| 元素 | 样式 |
|------|------|
| 代码块 | 深色背景 `#1e1e1e`，圆角 8px，橙色文字 |
| 内联代码 | 粉色 `#d19a66`，灰底 |
| 表格 | 交替行颜色，表头灰底 |
| 引用块 | 蓝色左边框 3px |
| 链接 | 蓝色 `#61afef` |
| 标题 | 渐变字号 h1~h4 |

### 6.4 入口 CSS 导入

```typescript
// src/main.tsx
import "katex/dist/katex.min.css";           // LaTeX 公式样式
import "highlight.js/styles/github-dark.css"; // 代码高亮暗色主题
```

### 6.5 提交记录

```
commit: Implement rich Markdown rendering with LaTeX and Obsidian Callout
- react-markdown with remark-gfm/math + rehype-katex/highlight
- Custom remark plugin for >[!note] callout detection
- 10 callout types with One Dark Pro semantic colors
- Full CSS: code blocks, tables, lists, blockquotes
```

---

## 七、One Dark Pro 主题

### 7.1 CSS 变量体系

```css
:root {
  --bg-primary:   #282c34;   /* 主内容区 */
  --bg-secondary: #21252b;   /* 面板/侧栏 */
  --bg-tertiary:  #1e1e1e;   /* 输入框/代码块 */
  --bg-hover:     #2c313a;   /* 悬停 */
  --text-primary: #abb2bf;   /* 主文字 */
  --text-secondary: #5c6370; /* 次要文字 */
  --text-muted:   #4b5263;   /* 禁用/提示 */
  --border:       #3a3f4b;   /* 边框 */
  --border-light: #2c313a;   /* 浅边框 */
  --accent:       #61afef;   /* 蓝色主色 */
  --success:      #98c379;
  --warning:      #e5c07b;
  --danger:       #e06c75;
  --purple:       #c678dd;
  --cyan:         #56b6c2;
  --orange:       #d19a66;
}
```

### 7.2 覆盖范围

所有 UI 元素均使用 CSS 变量，包括：
- 对话框 / 表单 / 按钮
- Markdown / 代码 / Callout
- 滚动条 / 状态栏
- highlight.js 覆盖 (`!important` 强制暗色背景)

---

## 八、当前文件结构（Phase 1 完成后）

```
D:\VScode Projects\Agent/
├── .gitignore
├── CLAUDE.md                       # 项目规则（含 Git 提交/推送规则）
├── DESKTOP_AGENT_PLAN.md           # 本文档
├── package.json
├── package-lock.json
├── index.html
├── vite.config.ts
├── tsconfig.json / .app.json / .node.json
├── dist/                           # 前端构建产物
├── src/
│   ├── main.tsx                    # 入口（导入 KaTeX + highlight.js CSS）
│   ├── App.tsx                     # 根组件
│   ├── App.css                     # 全局样式（One Dark Pro）
│   ├── vite-env.d.ts
│   ├── stores/
│   │   ├── settingsStore.ts        # 设置状态管理
│   │   ├── sessionStore.ts         # 会话 + Token 统计 + 发送消息
│   │   └── uiStore.ts              # 面板折叠状态
│   └── components/
│       ├── SessionList.tsx         # 左栏：会话列表
│       ├── ChatArea.tsx            # 中栏：对话 + 输入
│       ├── Dashboard.tsx           # 右栏：仪表盘（5 widgets）
│       ├── SettingsDialog.tsx      # 设置对话框
│       ├── StatusBar.tsx           # 底部状态栏
│       ├── MarkdownBlock.tsx       # Markdown 渲染（含 Callout 插件）
│       └── ObsidianCallout.tsx     # Obsidian Callout 组件
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/
│   │   ├── 32x32.png / 128x128.png / 128x128@2x.png
│   │   └── icon.ico
│   ├── gen/schemas/                # Tauri 自动生成
│   └── src/
│       ├── main.rs
│       ├── lib.rs                  # Tauri 配置 + 所有命令注册
│       ├── settings.rs             # 设置 CRUD + keyring 加密
│       ├── session.rs              # 会话 CRUD + JSON 持久化
│       └── deepseek.rs             # DeepSeek API 客户端
└── public/
```

---

## 九、Phase 2：Agent 核心（进行中）

### 9.1 Turn Loop 引擎

**目标**：实现 Agent 自主循环

```
loop {
    // 1. 构造请求（System Prompt + 历史消息 + 工具结果）
    // 2. 流式请求 DeepSeek API（stream=true）
    // 3. 解析 SSE 事件流
    match event {
        Token(text)          → 推送到前端渲染
        Reasoning(text)      → 显示思考过程
        ToolCall(name, args) → 执行工具 → 结果追加到上下文 → 继续循环
        Done                 → 返回最终结果
    }
}
```

**关键设计**：
- 系统 Prompt 包含角色定义 + 工具定义（JSON Schema）
- 工具结果作为新消息加入上下文，触发下一轮
- 上下文窗口管理：滑动窗口 + 摘要压缩

### 9.2 流式 SSE 接入

**Rust 端**：使用 `reqwest` 的流式 API 逐 chunk 读取 SSE
**前端**：通过 Tauri Event 系统实时接收 token

### 9.3 工具系统

**Tool trait**：
```rust
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;  // JSON Schema
    fn is_read_only(&self) -> bool;
    fn execute(&self, args: serde_json::Value) -> Result<String, String>;
}
```

**MVP 工具**：
| 工具 | 功能 | 只读 | 需审批 |
|------|------|------|--------|
| read | 读取文件 | ✅ | ❌ |
| write | 写入/覆盖 | ❌ | ✅ |
| edit | 精确替换 | ❌ | ✅ |
| glob | 文件搜索 | ✅ | ❌ |
| grep | 内容搜索 | ✅ | ❌ |
| bash | 命令执行 | 视情况 | 视情况 |
| finish | 结束任务 | — | — |

### 9.4 文件沙箱

- 限制文件操作到用户指定的工作目录
- 路径规范化，防止 `../` 逃逸
- Rust 侧路径校验

### 9.5 审批流程

- 只读工具 → 自动执行
- 写入工具 → 弹窗确认（可记忆为"本次会话允许"）
- Bash 命令 → 弹窗显示命令内容 + 确认

### 9.6 Diff 视图

- 文件修改后显示代码变更对比
- 绿色 = 新增，红色 = 删除
- 用户可以逐块接受/拒绝

---

## 十、完整构建验证

每次修改后执行：

```powershell
# 1. TypeScript 类型检查
npx tsc --noEmit

# 2. Vite 前端构建
npx vite build

# 3. Rust 后端编译
$env:PATH = "C:\Users\$env:USERNAME\.cargo\bin;$env:PATH"
cd src-tauri
cargo build

# 4. Tauri 完整构建（可选）
cd ..
npm run tauri build
```

---

## 十一、Git 工作流

```powershell
# 开发 → 提交（不要推送）
git add -A
git commit -m "description of changes"

# 仅在用户明确要求时推送
git push
```

---

## 十二、参考资源

| 项目 | 借鉴点 |
|------|--------|
| Claude Code | Turn Loop、工具系统、权限模型、上下文压缩 |
| OpenAI Codex | App Server 架构、三栏 UI、Thread/Turn 模型 |
| DeepSeek TUI (Rust) | 子 Agent、前缀缓存优化 |
| Cursor | 三栏布局、Diff 确认 |
