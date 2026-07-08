/** 对话主面板 — 中栏 */
export function ChatArea() {
  return (
    <main className="chat-area">
      {/* 对话消息区 */}
      <div className="messages">
        <div className="welcome">
          <h2>Agent PC</h2>
          <p>输入你的需求，AI 编码助手将帮你完成。</p>
        </div>
      </div>

      {/* 输入区 */}
      <div className="input-area">
        <div className="input-row">
          <input
            className="chat-input"
            type="text"
            placeholder="输入指令给 AI..."
            disabled
          />
          <button className="btn-send" disabled>
            发送
          </button>
        </div>
      </div>
    </main>
  );
}
