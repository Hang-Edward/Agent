import { useState, useRef, useEffect } from "react";
import { useSessionStore } from "../stores/sessionStore";
import { MarkdownBlock } from "./MarkdownBlock";

/** 对话主面板 — 中栏 */
export function ChatArea() {
  const {
    currentSession,
    currentId,
    loading,
    sending,
    streamContent,
    streamReasoning,
    lastError,
    sendMessage,
  } = useSessionStore();
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // 新消息时自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [currentSession?.messages.length, streamContent]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || !currentId || sending) return;
    setInput("");
    await sendMessage(text);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // 没有选中会话
  if (!currentId) {
    return (
      <main className="chat-area">
        <div className="messages">
          <div className="welcome">
            <h2>Agent PC</h2>
            <p>从左侧选择一个会话，或点击「新对话」开始。</p>
          </div>
        </div>
      </main>
    );
  }

  if (loading || !currentSession) {
    return (
      <main className="chat-area">
        <div className="messages">
          <div className="welcome"><p>加载中...</p></div>
        </div>
      </main>
    );
  }

  return (
    <main className="chat-area">
      {/* 消息列表 */}
      <div className="messages">
        {currentSession.messages.length === 0 && !sending ? (
          <div className="welcome">
            <h2>{currentSession.name}</h2>
            <p>输入你的需求，AI 编码助手将帮你完成。</p>
          </div>
        ) : (
          <>
            {currentSession.messages.map((msg) => (
              <div key={msg.id} className={`message message-${msg.role}`}>
                <div className="message-role">
                  {msg.role === "user" ? "你" : "AI"}
                </div>
                <div className="message-content">
                  {msg.role === "assistant" ? (
                    <MarkdownBlock content={msg.content} />
                  ) : (
                    msg.content
                  )}
                </div>
              </div>
            ))}

            {/* 错误提示 */}
            {lastError && (
              <div className="message message-error">
                <div className="message-role">错误</div>
                <div className="message-content">{lastError}</div>
              </div>
            )}

            {/* 流式回复 */}
            {sending && (
              <div className="message message-assistant">
                <div className="message-role">AI</div>
                <div className="message-content">
                  {/* 思考过程 */}
                  {streamReasoning && (
                    <details className="reasoning-block" open>
                      <summary>思考过程</summary>
                      <pre className="reasoning-text">{streamReasoning}</pre>
                    </details>
                  )}
                  {/* 流式文本 */}
                  {streamContent ? (
                    <MarkdownBlock content={streamContent} />
                  ) : (
                    <span className="thinking">思考中...</span>
                  )}
                </div>
              </div>
            )}
          </>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* 输入区 */}
      <div className="input-area">
        <div className="input-row">
          <input
            className="chat-input"
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={sending ? "等待 AI 回复..." : "输入指令给 AI..."}
            disabled={sending}
          />
          <button
            className="btn-send"
            onClick={handleSend}
            disabled={sending || !input.trim()}
          >
            发送
          </button>
        </div>
      </div>
    </main>
  );
}
