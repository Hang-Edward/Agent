import { useSessionStore } from "../stores/sessionStore";

/** 对话主面板 — 中栏 */
export function ChatArea() {
  const { currentSession, currentId, loading } = useSessionStore();

  // 没有选中会话 → 空状态
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
          <div className="welcome">
            <p>加载中...</p>
          </div>
        </div>
        <div className="input-area">
          <div className="input-row">
            <input className="chat-input" type="text" placeholder="加载中..." disabled />
            <button className="btn-send" disabled>发送</button>
          </div>
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
      </div>

      {/* 输入区 */}
      <div className="input-area">
        <div className="input-row">
          <input
            className="chat-input"
            type="text"
            placeholder="输入指令给 AI..."
          />
          <button className="btn-send">发送</button>
        </div>
      </div>
    </main>
  );
}
