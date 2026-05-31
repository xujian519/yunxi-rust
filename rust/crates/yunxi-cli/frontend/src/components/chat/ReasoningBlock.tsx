import type { FC } from 'react';
import { ChevronDown, Brain } from 'lucide-react';

interface ReasoningBlockProps {
  content: string;
  expanded: boolean;
  onToggle: () => void;
  isStreaming?: boolean;
}

const ReasoningBlock: FC<ReasoningBlockProps> = ({
  content,
  expanded,
  onToggle,
  isStreaming,
}) => {
  if (!content && !isStreaming) return null;

  return (
    <div
      style={{
        marginBottom: 8,
        borderRadius: 8,
        border: '1px dashed var(--border-secondary)',
        backgroundColor: 'var(--bg-surface)',
      }}
    >
      <button
        type="button"
        className="flex w-full items-center"
        style={{
          gap: 6,
          padding: '6px 10px',
          fontSize: 11,
          fontWeight: 500,
          color: 'var(--text-tertiary)',
          background: 'transparent',
          border: 'none',
          cursor: 'pointer',
        }}
        onClick={onToggle}
      >
        <Brain size={13} />
        <span>思考过程{isStreaming ? '…' : ''}</span>
        <ChevronDown
          size={12}
          style={{
            marginLeft: 'auto',
            transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: 'transform 0.15s',
          }}
        />
      </button>
      {expanded && content ? (
        <div
          style={{
            padding: '0 10px 10px',
            fontSize: 11,
            lineHeight: 1.55,
            color: 'var(--text-secondary)',
            whiteSpace: 'pre-wrap',
            maxHeight: 200,
            overflow: 'auto',
          }}
        >
          {content}
        </div>
      ) : null}
    </div>
  );
};

export default ReasoningBlock;
