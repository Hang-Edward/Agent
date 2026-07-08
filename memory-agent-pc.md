---
name: agent-pc-project
description: PC 桌面端 AI 编码 Agent 项目，Tauri 2 + React + DeepSeek API
metadata:
  type: project
---

## 项目状态 (2026-07-09)

**仓库**: https://github.com/Hang-Edward/Agent.git
**分支**: master

**已完成**:
- Tauri 2 项目骨架搭建完毕（Rust 后端 + React 前端）
- VS 2022 Build Tools 安装至 D:\应用-Applications\VS2022-BuildTools
- Rust 1.96.1 + MSVC 工具链配置完成
- npm 依赖安装完成
- 初始提交已推送至 GitHub

**关键配置**:
- 前端端口: 1420
- 窗口: 1280x800, 最小 900x600
- MSVC 位于: D:\应用-Applications\VS2022-BuildTools\VC\Tools\MSVC\14.44.35207

**下一个阶段**: Phase 1 — 三栏布局 UI + 对话面板 + DeepSeek API 接入

## 用户偏好

详见 CLAUDE.md:
- 对标 Codex UI 风格（三栏布局）
- Monaco Editor
- 三个权限级别（默认/自动审查/完全访问）
- Markdown + LaTeX + Obsidian Callout 渲染
- 代码中文注释
