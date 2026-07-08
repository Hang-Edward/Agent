import ReactMarkdown from "react-markdown";
import remarkMath from "remark-math";
import remarkGfm from "remark-gfm";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import { visit } from "unist-util-visit";
import type { Root } from "mdast";
import { ObsidianCallout, parseCallout } from "./ObsidianCallout";
import type { Components } from "react-markdown";

/* ───── Remark 插件：Obsidian Callout ───── */

/** 检测 blockquote 中的 `>[!type]` 语法，标记为 callout */
function remarkObsidianCallout() {
  return (tree: Root) => {
    visit(tree, "blockquote", (node: any) => {
      const firstPara = node.children?.[0];
      if (firstPara?.type !== "paragraph") return;
      const firstLeaf = firstPara.children?.[0];
      if (firstLeaf?.type !== "text") return;

      const match = parseCallout(firstLeaf.value);
      if (!match) return;

      // 移除 callout 标记行
      firstLeaf.value = match.rest;

      // 标记节点属性
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
  // Obsidian Callout
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

  // 代码块（带复制按钮）
  code: ({ className, children, ...props }) => {
    const isInline = !className;
    if (isInline) {
      return (
        <code className="md-code-inline" {...props}>
          {children}
        </code>
      );
    }
    return (
      <code className={className} {...props}>
        {children}
      </code>
    );
  },

  // 表格样式
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
  return (
    <ReactMarkdown
      components={components}
      remarkPlugins={[remarkObsidianCallout, remarkGfm, remarkMath]}
      rehypePlugins={[rehypeKatex, rehypeHighlight]}
    >
      {content}
    </ReactMarkdown>
  );
}
