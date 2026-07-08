import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "@xterm/xterm/css/xterm.css";

export function TerminalPanel() {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const [started, setStarted] = useState(false);

  useEffect(() => {
    if (!containerRef.current || started) return;

    const term = new Terminal({
      theme: {
        background: "#1e1e1e",
        foreground: "#abb2bf",
        cursor: "#528bff",
        selectionBackground: "#3a3f4b",
        black: "#1e1e1e",
        red: "#e06c75",
        green: "#98c379",
        yellow: "#e5c07b",
        blue: "#61afef",
        magenta: "#c678dd",
        cyan: "#56b6c2",
        white: "#abb2bf",
        brightBlack: "#5c6370",
        brightRed: "#e06c75",
        brightGreen: "#98c379",
        brightYellow: "#e5c07b",
        brightBlue: "#61afef",
        brightMagenta: "#c678dd",
        brightCyan: "#56b6c2",
        brightWhite: "#ffffff",
      },
      fontSize: 13,
      fontFamily: "'Cascadia Code', 'Fira Code', monospace",
      cursorBlink: true,
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.onData((data) => {
      invoke("terminal_write", { input: data }).catch(console.error);
    });

    // 监听终端输出
    const unlisten = listen<string>("terminal:output", (ev) => {
      term.write(ev.payload);
    });

    // 挂载并启动
    if (containerRef.current) {
      term.open(containerRef.current);
      fitAddon.fit();
    }

    terminalRef.current = term;
    setStarted(true);

    // 启动后端终端进程
    invoke("terminal_start").catch(console.error);

    // 响应窗口大小变化
    const resizeHandler = () => fitAddon.fit();
    window.addEventListener("resize", resizeHandler);

    return () => {
      window.removeEventListener("resize", resizeHandler);
      unlisten.then((fn) => fn());
      term.dispose();
      invoke("terminal_stop").catch(() => {});
    };
  }, []);

  return (
    <div className="terminal-wrapper">
      <div className="terminal-container" ref={containerRef} />
    </div>
  );
}
