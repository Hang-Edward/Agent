import ReactMarkdown from "react-markdown";
import remarkMath from "remark-math";
import remarkGfm from "remark-gfm";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import { visit } from "unist-util-visit";
import type { Root } from "mdast";
import { ObsidianCallout, parseCallout } from "./ObsidianCallout";
import type { Components } from "react-markdown";

/* ───── 预处理器：统一 LaTeX 定界符 ───── */

/**
 * 将 AI 输出的各种 LaTeX 定界符统一为 remark-math 能识别的 $ 格式：
 *   \(...\)  → $...$   (行内公式)
 *   \[...\]  → $$...$$  (行间公式)
 *   已有的 $...$ 和 $$...$$ 保持不变
 */
function normalizeLatexDelimiters(text: string): string {
  return text
    // \(...\) → $...$
    .replace(/\\\(/g, "$")
    .replace(/\\\)/g, "$")
    // \[...\] → $$...$$
    .replace(/\\\[/g, "$$")
    .replace(/\\\]/g, "$$");
}

/* ───── Remark 插件：Obsidian Callout ───── */

function remarkObsidianCallout() {
  return (tree: Root) => {
    visit(tree, "blockquote", (node: any) => {
      const firstPara = node.children?.[0];
      if (firstPara?.type !== "paragraph") return;
      const firstLeaf = firstPara.children?.[0];
      if (firstLeaf?.type !== "text") return;

      const match = parseCallout(firstLeaf.value);
      if (!match) return;

      firstLeaf.value = match.rest;
      node.data = {
        hProperties: {
          "data-callout": match.type,
          "data-title": match.title,
        },
      };
    });
  };
}

/* ───── 自定义渲染组件 ───── */

const components: Components = {
  blockquote: ({ children, ...props }) => {
    const attrs = props as Record<string, string>;
    if (attrs["data-callout"]) {
      return (
        <ObsidianCallout
          type={attrs["data-callout"]}
          title={attrs["data-title"] || undefined}
        >
          {children}
        </ObsidianCallout>
      );
    }
    return <blockquote className="md-blockquote">{children}</blockquote>;
  },

  code: ({ className, children, ...props }) => {
    if (!className) {
      return <code className="md-code-inline" {...props}>{children}</code>;
    }
    return <code className={className} {...props}>{children}</code>;
  },

  table: ({ children }) => (
    <div className="md-table-wrapper">
      <table className="md-table">{children}</table>
    </div>
  ),
};

/* ───── MarkdownBlock ───── */

interface Props {
  content: string;
}

export function MarkdownBlock({ content }: Props) {
  // 先预处理 LaTeX 定界符，再传递给 react-markdown
  const processed = normalizeLatexDelimiters(content);

  return (
    <ReactMarkdown
      components={components}
      remarkPlugins={[remarkObsidianCallout, remarkMath, remarkGfm]}
      rehypePlugins={[rehypeKatex, rehypeHighlight]}
    >
      {processed}
    </ReactMarkdown>
  );
}
