# Agent Desktop App

## 项目定位

PC 桌面端 AI 编码 Agent，对标 OpenAI Codex Desktop App，接入 DeepSeek API。

## 技术栈

- 桌面框架：Tauri 2（Rust 后端 + Web 前端）
- 前端：React 19 + TypeScript + Vite + Tailwind CSS + shadcn/ui
- 状态管理：Zustand
- 编辑器：Monaco Editor（效果优先，不在乎包体积）
- 终端：xterm.js + portable-pty
- Markdown 渲染：react-markdown + remark-math + rehype-katex + 自定义 Obsidian Callout 组件
- 持久化：SQLite
- 打包：msi/nsis 安装包

## UI 设计

- **布局**：对标 Codex 三栏布局（左栏导航 + 中栏对话 + 右栏预览/终端）
- **底部输入栏**：包含模式选择器、模型选择器、权限级别选择器
- **权限级别（三个）**：
  - 默认模式：高风险操作弹窗确认
  - 自动审查：自动执行，审查变更
  - 完全访问：YOLO 模式，一键放行
- **主题**：深色/浅色主题切换
- **Markdown 支持**：标准 Markdown + LaTeX 数学公式 + Obsidian Callout 语法（>[!note/info/warning/tip/...]）
- **编辑器**：Monaco Editor，语法高亮、自动补全、Diff 对比

## Agent 架构

- Agent Loop：while(tool_call) 循环（Claude Code 风格）
- 模型完全自主决策工具调用，不预设路径
- 工具系统：只读工具自动执行，写入工具需审批
- 上下文管理：滑动窗口 + LLM 压缩
- 子 Agent 系统（未来）：复杂任务分解

## DeepSeek API

- 模型：deepseek-v4-flash（默认） / deepseek-v4-pro（可切换）
- 流式 SSE 接入
- 支持 reasoning_content（思考过程展示）
- DSML 解析器处理 XML 工具调用
- 前缀缓存优化降本

## 开发偏好

- 大型软件安装在 D:\应用-Applications\
- 自动安装所需环境和依赖
- 效果优先，不在乎开发复杂度
- 代码要有中文注释
- 默认使用中文交流

## 阶段计划

### Phase 1：基础骨架
1. Tauri 2 项目初始化 + React + Tailwind + shadcn/ui
2. 三栏布局骨架
3. 对话 UI（气泡 + 输入框 + Markdown/LaTeX/Obsidian 渲染）
4. DeepSeek API 基础接入（非流式通断测试）
5. 设置页面（API Key + 模型选择 + 权限级别）
6. 基础会话管理（新建/切换/删除）

### Phase 2：Agent 核心
Turn Loop、工具系统、流式渲染、Diff 视图、审批流程

### Phase 3：增强功能
Bash 终端、Monaco Editor、上下文压缩、MCP 协议

### Phase 4：生产化
打包分发、自动更新、性能优化

### Phase 5：进阶特性
多 Agent、Skills、插件系统
