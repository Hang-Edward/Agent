import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Monaco Editor 独立分包（较大）
          "monaco": ["monaco-editor", "@monaco-editor/react"],
          // KaTeX + Markdown 分包
          "markdown": ["katex", "react-markdown", "remark-math", "rehype-katex", "rehype-highlight"],
          // xterm 分包
          "terminal": ["xterm", "@xterm/xterm", "@xterm/addon-fit"],
          // React 框架分包
          "vendor": ["react", "react-dom"],
        },
      },
    },
    chunkSizeWarningLimit: 500,
  },
});
