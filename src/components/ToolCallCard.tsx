/** 工具调用卡片：在对话中展示 AI 调用的工具和执行结果 */
interface ToolCallCardProps {
  name: string;
  args: string;
  result?: string;
  status: "running" | "success" | "error";
}

export function ToolCallCard({ name, args, result, status }: ToolCallCardProps) {
  let argsPreview = "";
  try {
    const parsed = JSON.parse(args);
    argsPreview = JSON.stringify(parsed, null, 2);
  } catch {
    argsPreview = args;
  }

  return (
    <div className={`tool-call tool-call-${status}`}>
      <div className="tool-call-header">
        <span className="tool-call-icon">
          {status === "running" && "⟳"}
          {status === "success" && "✓"}
          {status === "error" && "✕"}
        </span>
        <span className="tool-call-name">{name}</span>
        {status === "running" && <span className="tool-call-spinner" />}
      </div>
      <details className="tool-call-details" open={status === "error"}>
        <summary>参数</summary>
        <pre className="tool-call-args">{argsPreview}</pre>
      </details>
      {result && (
        <div className="tool-call-result">
          <pre className="tool-call-result-text">{result.slice(0, 500)}</pre>
        </div>
      )}
    </div>
  );
}
