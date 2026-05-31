import type { FC } from 'react';
import { ShieldAlert } from 'lucide-react';
import type { PendingPermission } from '@/data/mockData';

interface PermissionModalProps {
  pending: PendingPermission;
  onRespond: (outcome: 'allow' | 'deny' | 'always') => void;
}

const PermissionModal: FC<PermissionModalProps> = ({ pending, onRespond }) => {
  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center"
      style={{ backgroundColor: 'rgba(0,0,0,0.45)' }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="permission-title"
    >
      <div
        style={{
          width: 'min(420px, 92vw)',
          borderRadius: 12,
          border: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
          boxShadow: '0 16px 48px rgba(0,0,0,0.2)',
          padding: 20,
        }}
      >
        <div className="mb-3 flex items-center" style={{ gap: 10 }}>
          <ShieldAlert size={20} style={{ color: 'var(--status-warning)' }} />
          <h2 id="permission-title" style={{ fontSize: 15, fontWeight: 600, color: 'var(--text-primary)' }}>
            工具执行确认
          </h2>
        </div>
        <p style={{ fontSize: 13, color: 'var(--text-secondary)', marginBottom: 12 }}>
          云熙请求执行工具 <strong style={{ color: 'var(--accent-primary)' }}>{pending.tool}</strong>，是否允许？
        </p>
        <pre
          className="custom-scrollbar"
          style={{
            maxHeight: 160,
            overflow: 'auto',
            fontSize: 11,
            fontFamily: "'JetBrains Mono', monospace",
            padding: 10,
            borderRadius: 8,
            backgroundColor: 'var(--bg-surface)',
            border: '1px solid var(--border-primary)',
            color: 'var(--text-primary)',
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            marginBottom: 16,
          }}
        >
          {pending.input || '（无参数）'}
        </pre>
        <div className="flex justify-end" style={{ gap: 8 }}>
          <button
            type="button"
            onClick={() => onRespond('deny')}
            style={{
              padding: '8px 14px',
              fontSize: 13,
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'transparent',
              color: 'var(--text-secondary)',
            }}
          >
            拒绝
          </button>
          <button
            type="button"
            onClick={() => onRespond('always')}
            style={{
              padding: '8px 14px',
              fontSize: 13,
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-secondary)',
            }}
          >
            始终允许
          </button>
          <button
            type="button"
            onClick={() => onRespond('allow')}
            style={{
              padding: '8px 16px',
              fontSize: 13,
              fontWeight: 600,
              borderRadius: 8,
              border: 'none',
              backgroundColor: 'var(--accent-primary)',
              color: '#fff',
            }}
          >
            允许一次
          </button>
        </div>
      </div>
    </div>
  );
};

export default PermissionModal;
