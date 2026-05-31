import type { FC } from 'react';
import { useState } from 'react';
import { ChevronDown, Wrench, CheckCircle2, XCircle, Loader2 } from 'lucide-react';
import type { ToolCallBlock } from '@/data/mockData';

interface ToolCallCardProps {
  tool: ToolCallBlock;
}

const ToolCallCard: FC<ToolCallCardProps> = ({ tool }) => {
  const [expanded, setExpanded] = useState(false);

  const statusIcon =
    tool.status === 'running' ? (
      <Loader2 size={14} className="animate-spin" style={{ color: 'var(--accent-cyan)' }} />
    ) : tool.isError || tool.status === 'error' ? (
      <XCircle size={14} style={{ color: 'var(--status-error)' }} />
    ) : (
      <CheckCircle2 size={14} style={{ color: 'var(--status-success)' }} />
    );

  return (
    <div
      style={{
        marginTop: 8,
        borderRadius: 8,
        border: '1px solid var(--border-primary)',
        backgroundColor: 'var(--bg-surface)',
        overflow: 'hidden',
      }}
    >
      <button
        type="button"
        className="flex w-full items-center transition-colors"
        style={{
          gap: 8,
          padding: '8px 10px',
          fontSize: 12,
          color: 'var(--text-primary)',
          background: 'transparent',
          border: 'none',
          cursor: 'pointer',
        }}
        onClick={() => setExpanded((v) => !v)}
      >
        <Wrench size={13} style={{ color: 'var(--text-tertiary)', flexShrink: 0 }} />
        <span style={{ fontWeight: 600, flex: 1, textAlign: 'left' }}>{tool.name}</span>
        {statusIcon}
        <ChevronDown
          size={14}
          style={{
            color: 'var(--text-tertiary)',
            transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: 'transform 0.15s',
          }}
        />
      </button>
      {expanded && (
        <div style={{ padding: '0 10px 10px', fontSize: 11 }}>
          {tool.input ? (
            <div style={{ marginBottom: 6 }}>
              <div style={{ color: 'var(--text-tertiary)', marginBottom: 4 }}>输入</div>
              <pre
                style={{
                  margin: 0,
                  padding: 8,
                  borderRadius: 6,
                  backgroundColor: 'var(--bg-elevated)',
                  fontFamily: "'JetBrains Mono', monospace",
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-word',
                  maxHeight: 120,
                  overflow: 'auto',
                }}
              >
                {tool.input}
              </pre>
            </div>
          ) : null}
          {tool.output ? (
            <div>
              <div style={{ color: 'var(--text-tertiary)', marginBottom: 4 }}>输出</div>
              <pre
                style={{
                  margin: 0,
                  padding: 8,
                  borderRadius: 6,
                  backgroundColor: 'var(--bg-elevated)',
                  fontFamily: "'JetBrains Mono', monospace",
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-word',
                  maxHeight: 160,
                  overflow: 'auto',
                  color: tool.isError ? 'var(--status-error)' : 'var(--text-primary)',
                }}
              >
                {tool.output.length > 2000 ? `${tool.output.slice(0, 2000)}…` : tool.output}
              </pre>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
};

export default ToolCallCard;
