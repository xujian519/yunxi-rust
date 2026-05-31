import { memo } from 'react';
import type { PanelLogLine } from '@/types/workspace';

function formatLogs(panelLogs: PanelLogLine[]): string {
  if (panelLogs.length === 0) {
    return '输出为空。对话、检索与案件操作将记录在此。';
  }
  return panelLogs
    .map(
      (l) =>
        `[${l.time}] ${l.level.toUpperCase()} ${l.source ? `(${l.source}) ` : ''}${l.message}`,
    )
    .join('\n');
}

/** 与全局聊天流式更新隔离，避免输出区随 messages 重绘而闪烁 */
const OutputPanel = memo(function OutputPanel({ panelLogs }: { panelLogs: PanelLogLine[] }) {
  return (
    <pre
      className="custom-scrollbar m-0 h-full whitespace-pre-wrap break-words"
      style={{
        padding: 12,
        fontSize: 11,
        fontFamily: 'ui-monospace, monospace',
        color: 'var(--text-secondary)',
        lineHeight: 1.5,
        boxSizing: 'border-box',
      }}
    >
      {formatLogs(panelLogs)}
    </pre>
  );
});

export default OutputPanel;
