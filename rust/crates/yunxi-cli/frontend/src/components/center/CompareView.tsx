import type { FC } from 'react';
import { useMemo } from 'react';
import { motion } from 'framer-motion';
import { diffComparison } from '@/data/mockData';
import { GitCompare, AlertCircle } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import { isTauriRuntime } from '@/api';
import { buildSideBySideDiff, type DiffLine } from '@/utils/lineDiff';

const DiffLineRow: FC<{
  type: 'add' | 'del' | 'unchanged';
  content: string;
  lineNum: number;
}> = ({ type, content, lineNum }) => {
  const bgColor =
    type === 'add'
      ? 'rgba(74, 124, 111, 0.12)'
      : type === 'del'
        ? 'rgba(184, 92, 80, 0.12)'
        : 'transparent';
  const borderColor =
    type === 'add'
      ? 'var(--status-success)'
      : type === 'del'
        ? 'var(--status-error)'
        : 'transparent';
  const textColor =
    type === 'add'
      ? 'var(--status-success)'
      : type === 'del'
        ? 'var(--status-error)'
        : 'var(--text-primary)';
  const prefix = type === 'add' ? '+ ' : type === 'del' ? '- ' : '  ';

  return (
    <div
      className="flex"
      style={{
        backgroundColor: bgColor,
        borderLeft: `3px solid ${borderColor}`,
        minHeight: 24,
      }}
    >
      <div
        className="select-none text-right"
        style={{
          width: 40,
          paddingRight: 8,
          fontSize: 10,
          fontFamily: 'var(--editor-font-family)',
          color: 'var(--text-tertiary)',
          lineHeight: '24px',
          flexShrink: 0,
        }}
      >
        {lineNum}
      </div>
      <div
        style={{
          flex: 1,
          fontSize: 12,
          fontFamily: 'var(--editor-font-family)',
          lineHeight: '24px',
          color: textColor,
          paddingLeft: 8,
          whiteSpace: 'pre',
          overflow: 'visible',
        }}
      >
        {prefix}
        {content}
      </div>
    </div>
  );
};

const CompareView: FC = () => {
  const { getDocumentByType, activeCase } = useApp();

  const { original, modified, notice } = useMemo(() => {
    if (!isTauriRuntime()) {
      return {
        original: diffComparison.original,
        modified: diffComparison.modified,
        notice: null as string | null,
      };
    }

    const claimsDoc = getDocumentByType('claims') ?? activeCase?.documents.find((d) => d.type === 'claims');
    const draftsDoc = activeCase?.documents.find((d) => d.type === 'drafts');
    const leftText = claimsDoc?.contentMd ?? '';
    const rightText = draftsDoc?.contentMd ?? '';

    if (!leftText && !rightText) {
      return {
        original: [] as DiffLine[],
        modified: [] as DiffLine[],
        notice: '当前案件缺少权利要求与修改稿文档，请在左侧案件树中打开或刷新案件。',
      };
    }
    if (!draftsDoc) {
      return {
        original: [] as DiffLine[],
        modified: [] as DiffLine[],
        notice: '未找到 type=drafts 的修改稿文档；已尝试仅展示原始权利要求。',
      };
    }

    const diff = buildSideBySideDiff(leftText, rightText);
    return { original: diff.original, modified: diff.modified, notice: null };
  }, [getDocumentByType, activeCase]);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="flex h-full flex-col"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      <div
        className="flex items-center justify-between"
        style={{
          padding: '8px 16px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <div className="flex items-center" style={{ gap: 8 }}>
          <GitCompare size={14} style={{ color: 'var(--accent-primary)' }} />
          <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-primary)' }}>
            权利要求对比
          </span>
          {isTauriRuntime() && activeCase ? (
            <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>
              {activeCase.name}
            </span>
          ) : null}
        </div>
        <div className="flex items-center" style={{ gap: 12 }}>
          <div className="flex items-center" style={{ gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: 2, backgroundColor: 'var(--status-success)' }} />
            <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>新增</span>
          </div>
          <div className="flex items-center" style={{ gap: 4 }}>
            <div style={{ width: 8, height: 8, borderRadius: 2, backgroundColor: 'var(--status-error)' }} />
            <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>删除</span>
          </div>
        </div>
      </div>

      {notice ? (
        <div
          className="flex items-start"
          style={{ gap: 8, padding: 16, fontSize: 12, color: 'var(--status-warning)' }}
        >
          <AlertCircle size={16} style={{ flexShrink: 0 }} />
          <span>{notice}</span>
        </div>
      ) : null}

      <div className="flex flex-1 overflow-hidden">
        <div
          className="flex-1 overflow-auto custom-scrollbar"
          style={{ borderRight: '1px solid var(--border-primary)' }}
        >
          <div
            className="sticky top-0"
            style={{
              padding: '6px 12px',
              fontSize: 11,
              fontWeight: 600,
              color: 'var(--text-secondary)',
              backgroundColor: 'var(--bg-elevated)',
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            原始版本（claims）
          </div>
          {original.map((line, idx) => (
            <DiffLineRow
              key={`orig-${idx}`}
              type={line.type}
              content={line.content}
              lineNum={line.lineNum}
            />
          ))}
        </div>

        <div className="flex-1 overflow-auto custom-scrollbar">
          <div
            className="sticky top-0"
            style={{
              padding: '6px 12px',
              fontSize: 11,
              fontWeight: 600,
              color: 'var(--text-secondary)',
              backgroundColor: 'var(--bg-elevated)',
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            修改版本（drafts）
          </div>
          {modified.map((line, idx) => (
            <DiffLineRow
              key={`mod-${idx}`}
              type={line.type}
              content={line.content}
              lineNum={line.lineNum}
            />
          ))}
        </div>
      </div>
    </motion.div>
  );
};

export default CompareView;
