/** 审批弹窗：高风险操作（写文件、执行命令）时弹出确认 */
interface PendingAction {
  id: string;
  tool: string;
  description: string;
  args: string;
}

interface ApprovalDialogProps {
  pending: PendingAction | null;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  onAlwaysAllow: (tool: string) => void;
}

export function ApprovalDialog({
  pending,
  onApprove,
  onReject,
  onAlwaysAllow,
}: ApprovalDialogProps) {
  if (!pending) return null;

  return (
    <div className="overlay" onClick={() => onReject(pending.id)}>
      <div className="dialog approval-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>🔒 操作确认</h2>
        </div>
        <div className="dialog-body">
          <p className="approval-desc">
            <strong>{pending.tool}</strong> 请求执行：
          </p>
          <pre className="approval-args">{pending.args}</pre>
        </div>
        <div className="dialog-footer approval-footer">
          <button className="btn" onClick={() => onReject(pending.id)}>
            拒绝
          </button>
          <button className="btn" onClick={() => onAlwaysAllow(pending.tool)}>
            本次允许
          </button>
          <button
            className="btn btn-primary"
            onClick={() => onApprove(pending.id)}
          >
            允许
          </button>
        </div>
      </div>
    </div>
  );
}
