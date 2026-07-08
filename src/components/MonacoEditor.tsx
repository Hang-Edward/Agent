import Editor, { type OnMount } from "@monaco-editor/react";
import { useRef } from "react";
import type { editor } from "monaco-editor";

interface Props {
  path: string;
  content: string;
  language?: string;
  onChange?: (value: string | undefined) => void;
  readOnly?: boolean;
}

/** 根据文件扩展名推断语言 */
function inferLanguage(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() || "";
  const map: Record<string, string> = {
    ts: "typescript",
    tsx: "typescript",
    js: "javascript",
    jsx: "javascript",
    rs: "rust",
    py: "python",
    rs: "rust",
    json: "json",
    md: "markdown",
    css: "css",
    html: "html",
    xml: "xml",
    yaml: "yaml",
    yml: "yaml",
    toml: "ini",
    sql: "sql",
    sh: "shell",
    ps1: "powershell",
    bat: "bat",
    lock: "json",
    gitignore: "plaintext",
  };
  return map[ext] || "plaintext";
}

export function MonacoEditor({
  path,
  content,
  language,
  onChange,
  readOnly = true,
}: Props) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;
  };

  return (
    <div className="monaco-wrapper">
      <Editor
        key={path}
        defaultLanguage={language || inferLanguage(path)}
        defaultValue={content}
        theme="vs-dark"
        onChange={onChange}
        onMount={handleMount}
        options={{
          readOnly,
          minimap: { enabled: false },
          fontSize: 13,
          fontFamily: "'Cascadia Code', 'Fira Code', monospace",
          lineNumbers: "on",
          scrollBeyondLastLine: false,
          automaticLayout: true,
          tabSize: 2,
          wordWrap: "on",
          renderWhitespace: "selection",
          padding: { top: 8 },
        }}
      />
    </div>
  );
}
