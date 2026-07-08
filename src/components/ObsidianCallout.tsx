import type { ReactNode } from "react";

/** Obsidian 支持的 Callout 类型 */
const CALLOUT_TYPES = [
  "note", "info", "tip", "warning", "danger",
  "success", "question", "bug", "example", "quote",
] as const;

type CalloutType = (typeof CALLOUT_TYPES)[number];

/** Callout 类型 → 显示名 + 颜色变量 */
const CALLOUT_META: Record<
  CalloutType,
  { label: string; icon: string }
> = {
  note:    { label: "笔记",   icon: "📝" },
  info:    { label: "信息",   icon: "ℹ️" },
  tip:     { label: "提示",   icon: "💡" },
  warning: { label: "注意",   icon: "⚠️" },
  danger:  { label: "危险",   icon: "🔥" },
  success: { label: "成功",   icon: "✅" },
  question:{ label: "疑问",   icon: "❓" },
  bug:     { label: "缺陷",   icon: "🐛" },
  example: { label: "示例",   icon: "📎" },
  quote:   { label: "引用",   icon: "💬" },
};

interface Props {
  type: string;
  title?: string;
  children: ReactNode;
}

/** Obsidian 风格的 Callout 块 */
export function ObsidianCallout({ type, title, children }: Props) {
  const t = type.toLowerCase() as CalloutType;
  const meta = CALLOUT_META[t] ?? { label: type, icon: "📌" };

  return (
    <div className={`callout callout-${t}`}>
      <div className="callout-header">
        <span className="callout-icon">{meta.icon}</span>
        <span className="callout-label">{title || meta.label}</span>
      </div>
      <div className="callout-body">{children}</div>
    </div>
  );
}

/** 检查 blockquote 文本是否以 Obsidian callout 开头 */
export function parseCallout(
  text: string,
): { type: string; title: string; rest: string } | null {
  const match = text.match(/^\s*\[!(\w+)\]\s*(.*?)(?:\n|$)/);
  if (!match) return null;

  const type = match[1].toLowerCase();
  if (!CALLOUT_TYPES.includes(type as CalloutType)) return null;

  return {
    type,
    title: match[2].trim(),
    rest: text.slice(match[0].length),
  };
}
