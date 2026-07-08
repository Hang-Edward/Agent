import { useState } from "react";
import { useSessionStore } from "../stores/sessionStore";

/** 对话主面板 — 中栏 */
export function ChatArea() {
  const { currentSession, currentId, loading, sending, sendMessage } =
    useSessionStore();
  const [input, setInput] = useState("");

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

  // 加载中
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
        {currentSession.messages.length === 0 ? (
          <div className="welcome">
            <h2>{currentSession.name}</h2>
            <p>输入你的需求，AI 编码助手将帮你完成。</p>
          </div>
        ) : (
          currentSession.messages.map((msg) => (
            <div key={msg.id} className={`message message-${msg.role}`}>
              <div className="message-role">
                {msg.role === "user" ? "你" : "AI"}
              </div>
              <div className="message-content">{msg.content}</div>
            </div>
          ))
        )}
        {/* 发送中指示器 */}
        {sending && (
          <div className="message message-assistant">
            <div className="message-role">AI</div>
            <div className="message-content thinking">思考中...</div>
          </div>
        )}
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
