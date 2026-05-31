import type { FC } from 'react';
import { useEffect, useRef, useState } from 'react';
import { motion } from 'framer-motion';
import {
  Bold,
  Italic,
  Heading,
  List,
  ListOrdered,
  Quote,
  FileText,
  Save,
  Undo,
  Redo,
} from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import { isTauriRuntime } from '@/api';

const ToolbarButton: FC<{
  icon: LucideIcon;
  onClick?: () => void;
  active?: boolean;
  title?: string;
}> = ({ icon: Icon, onClick, active, title }) => (
  <button
    onClick={onClick}
    title={title}
    className="flex items-center justify-center transition-all duration-150"
    style={{
      width: 28,
      height: 28,
      borderRadius: 5,
      backgroundColor: active ? 'var(--accent-primary-muted)' : 'transparent',
      color: active ? 'var(--accent-primary)' : 'var(--text-tertiary)',
    }}
    type="button"
  >
    <Icon size={14} />
  </button>
);

const DEFAULT_DRAFT = `技术领域

本发明涉及电池管理技术领域，具体涉及一种基于深度学习的智能电池管理系统。

背景技术

随着新能源汽车和储能系统的快速发展，锂离子电池的安全性和使用寿命越来越受到关注。

发明内容

本发明的目的在于提供一种智能电池管理系统，能够实时监测电池状态并准确预测电池退化趋势。`;

const DraftView: FC = () => {
  const { getDocumentByType, updateCaseDocument, activeCase } = useApp();
  const descDoc = getDocumentByType('description');
  const docId = descDoc?.id;

  const [content, setContent] = useState(descDoc?.contentMd || DEFAULT_DRAFT);
  const [isSaved, setIsSaved] = useState(true);
  const [saveError, setSaveError] = useState<string | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setContent(descDoc?.contentMd || DEFAULT_DRAFT);
    setIsSaved(true);
    setSaveError(null);
  }, [descDoc?.id, descDoc?.contentMd, activeCase?.id]);

  const scheduleSave = (next: string) => {
    if (!isTauriRuntime() || !docId) {
      setIsSaved(true);
      return;
    }
    setIsSaved(false);
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      void updateCaseDocument(docId, next)
        .then(() => {
          setIsSaved(true);
          setSaveError(null);
        })
        .catch((e) => {
          setSaveError(e instanceof Error ? e.message : String(e));
          setIsSaved(false);
        });
    }, 1200);
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const next = e.target.value;
    setContent(next);
    scheduleSave(next);
  };

  const handleSaveNow = () => {
    if (!docId) return;
    void updateCaseDocument(docId, content)
      .then(() => {
        setIsSaved(true);
        setSaveError(null);
      })
      .catch((e) => setSaveError(e instanceof Error ? e.message : String(e)));
  };

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
          padding: '6px 12px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <div className="flex items-center" style={{ gap: 2 }}>
          <ToolbarButton icon={Bold} title="粗体（待接编辑器）" />
          <ToolbarButton icon={Italic} title="斜体" />
          <div style={{ width: 1, height: 18, backgroundColor: 'var(--border-primary)', margin: '0 6px' }} />
          <ToolbarButton icon={Heading} title="标题" />
          <ToolbarButton icon={List} title="无序列表" />
          <ToolbarButton icon={ListOrdered} title="有序列表" />
          <div style={{ width: 1, height: 18, backgroundColor: 'var(--border-primary)', margin: '0 6px' }} />
          <ToolbarButton icon={Quote} title="引用" />
          <ToolbarButton icon={FileText} title="权利要求引用" />
          <div style={{ width: 1, height: 18, backgroundColor: 'var(--border-primary)', margin: '0 6px' }} />
          <ToolbarButton icon={Undo} title="撤销" />
          <ToolbarButton icon={Redo} title="重做" />
        </div>

        <div className="flex items-center" style={{ gap: 8 }}>
          {saveError ? (
            <span style={{ fontSize: 10, color: 'var(--status-error)' }}>{saveError}</span>
          ) : null}
          <motion.div
            animate={isSaved ? { scale: [1, 1.2, 1] } : {}}
            transition={{ duration: 0.3 }}
            style={{
              width: 6,
              height: 6,
              borderRadius: '50%',
              backgroundColor: isSaved ? 'var(--status-success)' : 'var(--status-warning)',
            }}
          />
          <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>
            {isTauriRuntime()
              ? isSaved
                ? '已保存到案件'
                : '保存中…'
              : 'Mock 模式（不持久化）'}
          </span>
          {isTauriRuntime() && docId ? (
            <button type="button" onClick={handleSaveNow} title="立即保存" style={{ border: 'none', background: 'transparent', padding: 0 }}>
              <Save size={12} style={{ color: 'var(--accent-primary)' }} />
            </button>
          ) : (
            <Save size={12} style={{ color: 'var(--text-tertiary)' }} />
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto custom-scrollbar">
        <textarea
          value={content}
          onChange={handleChange}
          className="h-full w-full resize-none bg-transparent focus:outline-none"
          style={{
            padding: '20px 24px',
            fontSize: 14,
            lineHeight: 1.7,
            color: 'var(--text-primary)',
            fontFamily: "'Inter', system-ui, sans-serif",
            minHeight: '100%',
          }}
          spellCheck={false}
        />
      </div>
    </motion.div>
  );
};

export default DraftView;
