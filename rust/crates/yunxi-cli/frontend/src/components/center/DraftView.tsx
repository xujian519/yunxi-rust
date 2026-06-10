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
  PenSquare,
  ChevronDown,
} from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import { isTauriRuntime } from '@/api';
import { tauriApi } from '@/api/tauri';

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

  // Drafting dialog state
  const [draftDialog, setDraftDialog] = useState<'claim' | 'abstract' | 'spec' | null>(null);
  const [showDraftMenu, setShowDraftMenu] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [generateError, setGenerateError] = useState<string | null>(null);
  const draftMenuRef = useRef<HTMLDivElement | null>(null);

  // Claim generator form
  const [claimFeatures, setClaimFeatures] = useState('');
  const [claimType, setClaimType] = useState('independent');
  const [claimScope, setClaimScope] = useState('broad');

  // Abstract drafter form
  const [absTitle, setAbsTitle] = useState('');
  const [absField, setAbsField] = useState('');
  const [absProblem, setAbsProblem] = useState('');
  const [absSolution, setAbsSolution] = useState('');

  // Specification drafter form
  const [specClaims, setSpecClaims] = useState('');
  const [specAbstract, setSpecAbstract] = useState('');
  const [specField, setSpecField] = useState('');
  const [specBg, setSpecBg] = useState('');
  const [specDesc, setSpecDesc] = useState('');

  useEffect(() => {
    setContent(descDoc?.contentMd || DEFAULT_DRAFT);
    setIsSaved(true);
    setSaveError(null);
  }, [descDoc?.id, descDoc?.contentMd, activeCase?.id]);

  // Close draft menu on outside click
  useEffect(() => {
    if (!showDraftMenu) return;
    const handleClick = (e: MouseEvent) => {
      if (draftMenuRef.current && !draftMenuRef.current.contains(e.target as Node)) {
        setShowDraftMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [showDraftMenu]);

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

  const openDialog = (type: 'claim' | 'abstract' | 'spec') => {
    setShowDraftMenu(false);
    setGenerateError(null);
    setDraftDialog(type);
  };

  const closeDialog = () => {
    setDraftDialog(null);
    setGenerateError(null);
    setIsGenerating(false);
  };

  const handleClaimGenerate = async () => {
    if (!isTauriRuntime()) {
      setGenerateError('仅 Tauri 模式下可用');
      return;
    }
    if (!claimFeatures.trim()) {
      setGenerateError('请输入技术特征描述');
      return;
    }
    setIsGenerating(true);
    setGenerateError(null);
    try {
      const result = await tauriApi.claimGenerator(claimFeatures, claimType || undefined, claimScope || undefined);
      setContent(result);
      setIsSaved(false);
      closeDialog();
    } catch (e) {
      setGenerateError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsGenerating(false);
    }
  };

  const handleAbstractGenerate = async () => {
    if (!isTauriRuntime()) {
      setGenerateError('仅 Tauri 模式下可用');
      return;
    }
    if (!absTitle.trim() || !absField.trim() || !absProblem.trim() || !absSolution.trim()) {
      setGenerateError('请填写所有必填字段');
      return;
    }
    setIsGenerating(true);
    setGenerateError(null);
    try {
      const result = await tauriApi.abstractDrafter(absTitle, absField, absProblem, absSolution);
      setContent(result);
      setIsSaved(false);
      closeDialog();
    } catch (e) {
      setGenerateError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsGenerating(false);
    }
  };

  const handleSpecGenerate = async () => {
    if (!isTauriRuntime()) {
      setGenerateError('仅 Tauri 模式下可用');
      return;
    }
    if (!specClaims.trim() || !specAbstract.trim() || !specField.trim()) {
      setGenerateError('请填写所有必填字段');
      return;
    }
    setIsGenerating(true);
    setGenerateError(null);
    try {
      const result = await tauriApi.specificationDrafter(
        specClaims, specAbstract, specField,
        specBg || undefined, specDesc || undefined
      );
      setContent(result);
      setIsSaved(false);
      closeDialog();
    } catch (e) {
      setGenerateError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsGenerating(false);
    }
  };

  const labelStyle: React.CSSProperties = {
    display: 'block',
    fontSize: 12,
    color: 'var(--text-secondary)',
    marginBottom: 4,
  };

  const inputStyle: React.CSSProperties = {
    width: '100%',
    padding: '6px 10px',
    borderRadius: 6,
    border: '1px solid var(--border-primary)',
    backgroundColor: 'var(--bg-surface)',
    color: 'var(--text-primary)',
    fontSize: 13,
    outline: 'none',
    boxSizing: 'border-box',
  };

  const selectStyle: React.CSSProperties = {
    ...inputStyle,
    cursor: 'pointer',
  };

  const renderDialog = () => {
    if (!draftDialog) return null;

    const dialogProps = {
      onClose: closeDialog,
      loading: isGenerating,
      error: generateError,
    };

    if (draftDialog === 'claim') {
      return (
        <DraftDialog title="生成权利要求" onSubmit={handleClaimGenerate} {...dialogProps}>
          <label style={labelStyle}>技术特征描述 *</label>
          <textarea
            value={claimFeatures}
            onChange={(e) => setClaimFeatures(e.target.value)}
            rows={6}
            placeholder="描述发明的技术特征，如：一种智能电池管理系统，包括采集模块、分析模块和控制模块…"
            style={{ ...inputStyle, resize: 'vertical' }}
          />
          <div style={{ display: 'flex', gap: 12, marginTop: 12 }}>
            <div style={{ flex: 1 }}>
              <label style={labelStyle}>权利要求类型</label>
              <select value={claimType} onChange={(e) => setClaimType(e.target.value)} style={selectStyle}>
                <option value="independent">独立权利要求</option>
                <option value="dependent">从属权利要求</option>
              </select>
            </div>
            <div style={{ flex: 1 }}>
              <label style={labelStyle}>保护范围</label>
              <select value={claimScope} onChange={(e) => setClaimScope(e.target.value)} style={selectStyle}>
                <option value="broad">宽泛</option>
                <option value="medium">适中</option>
                <option value="narrow">窄</option>
              </select>
            </div>
          </div>
        </DraftDialog>
      );
    }

    if (draftDialog === 'abstract') {
      return (
        <DraftDialog title="生成摘要" onSubmit={handleAbstractGenerate} {...dialogProps}>
          <label style={labelStyle}>发明名称 *</label>
          <input value={absTitle} onChange={(e) => setAbsTitle(e.target.value)} placeholder="如：一种智能电池管理系统" style={inputStyle} />
          <div style={{ marginTop: 12 }} />
          <label style={labelStyle}>技术领域 *</label>
          <input value={absField} onChange={(e) => setAbsField(e.target.value)} placeholder="如：电池管理技术" style={inputStyle} />
          <div style={{ marginTop: 12 }} />
          <label style={labelStyle}>技术问题 *</label>
          <input value={absProblem} onChange={(e) => setAbsProblem(e.target.value)} placeholder="要解决的技术问题" style={inputStyle} />
          <div style={{ marginTop: 12 }} />
          <label style={labelStyle}>技术方案 *</label>
          <textarea
            value={absSolution}
            onChange={(e) => setAbsSolution(e.target.value)}
            rows={4}
            placeholder="技术方案的简要描述"
            style={{ ...inputStyle, resize: 'vertical' }}
          />
        </DraftDialog>
      );
    }

    if (draftDialog === 'spec') {
      return (
        <DraftDialog title="生成说明书" onSubmit={handleSpecGenerate} {...dialogProps}>
          <label style={labelStyle}>权利要求 *</label>
          <textarea
            value={specClaims}
            onChange={(e) => setSpecClaims(e.target.value)}
            rows={5}
            placeholder="粘贴权利要求全文，作为说明书撰写的依据"
            style={{ ...inputStyle, resize: 'vertical' }}
          />
          <div style={{ marginTop: 12 }} />
          <label style={labelStyle}>摘要 *</label>
          <textarea
            value={specAbstract}
            onChange={(e) => setSpecAbstract(e.target.value)}
            rows={3}
            placeholder="粘贴专利摘要"
            style={{ ...inputStyle, resize: 'vertical' }}
          />
          <div style={{ display: 'flex', gap: 12, marginTop: 12 }}>
            <div style={{ flex: 1 }}>
              <label style={labelStyle}>技术领域 *</label>
              <input value={specField} onChange={(e) => setSpecField(e.target.value)} placeholder="如：电池管理技术" style={inputStyle} />
            </div>
          </div>
          <div style={{ marginTop: 12 }} />
          <label style={labelStyle}>背景技术（可选）</label>
          <textarea
            value={specBg}
            onChange={(e) => setSpecBg(e.target.value)}
            rows={3}
            placeholder="现有技术的不足和待解决问题"
            style={{ ...inputStyle, resize: 'vertical' }}
          />
          <div style={{ marginTop: 12 }} />
          <label style={labelStyle}>具体实施方式（可选）</label>
          <textarea
            value={specDesc}
            onChange={(e) => setSpecDesc(e.target.value)}
            rows={4}
            placeholder="已有实施方式描述或思路"
            style={{ ...inputStyle, resize: 'vertical' }}
          />
        </DraftDialog>
      );
    }

    return null;
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
          <div ref={draftMenuRef} style={{ position: 'relative' }}>
            <button
              onClick={() => setShowDraftMenu((v) => !v)}
              className="flex items-center gap-1 transition-all duration-150"
              style={{
                padding: '4px 8px',
                borderRadius: 6,
                fontSize: 12,
                color: showDraftMenu ? 'var(--accent-primary)' : 'var(--text-secondary)',
                backgroundColor: showDraftMenu ? 'var(--bg-hover)' : 'transparent',
                border: 'none',
                cursor: 'pointer',
                whiteSpace: 'nowrap',
              }}
              type="button"
              title="专利撰写工具"
            >
              <PenSquare size={14} />
              <span>撰写</span>
              <ChevronDown size={12} style={{ transform: showDraftMenu ? 'rotate(180deg)' : undefined, transition: 'transform 0.15s' }} />
            </button>
            {showDraftMenu && (
              <div
                className="absolute top-full left-0 mt-1 z-50 rounded-lg shadow-lg overflow-hidden"
                style={{
                  backgroundColor: 'var(--bg-elevated)',
                  border: '1px solid var(--border-primary)',
                  minWidth: 160,
                }}
              >
                <button
                  onClick={() => openDialog('claim')}
                  className="hover:bg-[var(--bg-hover)]"
                  style={{ width: '100%', padding: '8px 12px', fontSize: 13, textAlign: 'left', border: 'none', background: 'transparent', color: 'var(--text-primary)', cursor: 'pointer', display: 'block' }}
                  type="button"
                >
                  生成权利要求
                </button>
                <button
                  onClick={() => openDialog('abstract')}
                  className="hover:bg-[var(--bg-hover)]"
                  style={{ width: '100%', padding: '8px 12px', fontSize: 13, textAlign: 'left', border: 'none', background: 'transparent', color: 'var(--text-primary)', cursor: 'pointer', display: 'block' }}
                  type="button"
                >
                  生成摘要
                </button>
                <button
                  onClick={() => openDialog('spec')}
                  className="hover:bg-[var(--bg-hover)]"
                  style={{ width: '100%', padding: '8px 12px', fontSize: 13, textAlign: 'left', border: 'none', background: 'transparent', color: 'var(--text-primary)', cursor: 'pointer', display: 'block' }}
                  type="button"
                >
                  生成说明书
                </button>
              </div>
            )}
          </div>
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

      {renderDialog()}
    </motion.div>
  );
};

const DraftDialog: FC<{
  title: string;
  children: React.ReactNode;
  onClose: () => void;
  onSubmit: () => void;
  loading: boolean;
  error: string | null;
}> = ({ title, children, onClose, onSubmit, loading, error }) => (
  <div
    className="fixed inset-0 z-50 flex items-center justify-center"
    style={{ backgroundColor: 'rgba(0,0,0,0.4)' }}
    onClick={onClose}
  >
    <div
      className="rounded-xl shadow-xl max-h-[80vh] overflow-y-auto"
      style={{
        width: 520,
        padding: 24,
        backgroundColor: 'var(--bg-surface)',
        border: '1px solid var(--border-primary)',
      }}
      onClick={(e) => e.stopPropagation()}
    >
      <h3 style={{ fontSize: 15, fontWeight: 600, color: 'var(--text-primary)', marginBottom: 16 }}>{title}</h3>
      {children}
      {error ? (
        <p style={{ fontSize: 12, color: 'var(--status-error)', marginTop: 8 }}>{error}</p>
      ) : null}
      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 16 }}>
        <button
          onClick={onClose}
          disabled={loading}
          style={{
            padding: '6px 16px',
            borderRadius: 6,
            border: '1px solid var(--border-primary)',
            backgroundColor: 'transparent',
            color: 'var(--text-secondary)',
            fontSize: 13,
            cursor: loading ? 'not-allowed' : 'pointer',
            opacity: loading ? 0.5 : 1,
          }}
          type="button"
        >
          取消
        </button>
        <button
          onClick={onSubmit}
          disabled={loading}
          style={{
            padding: '6px 16px',
            borderRadius: 6,
            border: 'none',
            backgroundColor: 'var(--accent-primary)',
            color: '#fff',
            fontSize: 13,
            cursor: loading ? 'not-allowed' : 'pointer',
            opacity: loading ? 0.7 : 1,
          }}
          type="button"
        >
          {loading ? '生成中…' : '生成'}
        </button>
      </div>
    </div>
  </div>
);

export default DraftView;
