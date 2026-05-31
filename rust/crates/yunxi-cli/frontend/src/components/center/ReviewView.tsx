import type { FC } from 'react';
import { useCallback, useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ChevronDown,
  AlertTriangle,
  Lightbulb,
  FileText,
  CheckCircle,
  BookOpen,
  Loader2,
} from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import { api, isTauriRuntime } from '@/api';
import {
  defaultReviewData,
  parseReviewDocument,
  serializeReviewData,
  type ReviewData,
  type ReviewResponse,
} from '@/utils/reviewParse';

const ReviewView: FC = () => {
  const { getDocumentByType, updateCaseDocument, activeCase } = useApp();
  const reviewDoc = getDocumentByType('review');

  const [data, setData] = useState<ReviewData>(() => defaultReviewData());
  const [expandedObjections, setExpandedObjections] = useState<Set<string>>(new Set(['obj-1']));
  const [knowledgeLoading, setKnowledgeLoading] = useState(false);
  const [knowledgeNote, setKnowledgeNote] = useState<string | null>(null);

  useEffect(() => {
    if (reviewDoc?.contentMd) {
      setData(parseReviewDocument(reviewDoc.contentMd));
      const first = parseReviewDocument(reviewDoc.contentMd).objections[0]?.id;
      if (first) setExpandedObjections(new Set([first]));
    } else if (!isTauriRuntime()) {
      setData(defaultReviewData());
    } else {
      setData({ objections: [], responses: [] });
    }
  }, [reviewDoc?.id, reviewDoc?.contentMd, activeCase?.id]);

  const persistReview = useCallback(
    async (next: ReviewData) => {
      if (!isTauriRuntime() || !reviewDoc?.id) return;
      await updateCaseDocument(reviewDoc.id, serializeReviewData(next));
    },
    [reviewDoc?.id, updateCaseDocument],
  );

  const toggleObjection = (id: string) => {
    setExpandedObjections((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const updateResponse = (objectionId: string, content: string) => {
    setData((prev) => {
      const existing = prev.responses.find((r) => r.objectionId === objectionId);
      let responses: ReviewResponse[];
      if (existing) {
        responses = prev.responses.map((r) =>
          r.objectionId === objectionId ? { ...r, content } : r,
        );
      } else {
        responses = [
          ...prev.responses,
          {
            id: `resp-${objectionId}`,
            objectionId,
            content,
          },
        ];
      }
      const next = { ...prev, responses };
      void persistReview(next);
      return next;
    });
  };

  const runKnowledgeHint = async () => {
    if (!isTauriRuntime()) {
      setKnowledgeNote('（Mock）知识库检索需在桌面客户端运行');
      return;
    }
    const first = data.objections[0];
    if (!first) return;
    setKnowledgeLoading(true);
    setKnowledgeNote(null);
    try {
      const q = `${first.type} ${first.claim} ${first.content.slice(0, 80)}`;
      const raw = await api.knowledgeSearch(q);
      setKnowledgeNote(raw.length > 1200 ? `${raw.slice(0, 1200)}…` : raw);
    } catch (e) {
      setKnowledgeNote(e instanceof Error ? e.message : String(e));
    } finally {
      setKnowledgeLoading(false);
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'novelty':
        return <AlertTriangle size={14} style={{ color: 'var(--status-error)' }} />;
      case 'inventive':
        return <Lightbulb size={14} style={{ color: 'var(--status-warning)' }} />;
      case 'support':
        return <FileText size={14} style={{ color: 'var(--status-info)' }} />;
      default:
        return <AlertTriangle size={14} style={{ color: 'var(--status-error)' }} />;
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'novelty':
        return '新颖性';
      case 'inventive':
        return '创造性';
      case 'support':
        return '支持问题';
      default:
        return type;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'novelty':
        return 'var(--status-error)';
      case 'inventive':
        return 'var(--status-warning)';
      case 'support':
        return 'var(--status-info)';
      default:
        return 'var(--text-tertiary)';
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="h-full overflow-auto custom-scrollbar"
      style={{ backgroundColor: 'var(--bg-surface)', padding: 20 }}
    >
      <div className="mb-5 flex items-start justify-between">
        <div>
          <h2
            style={{
              fontSize: 18,
              fontWeight: 600,
              color: 'var(--text-primary)',
              letterSpacing: '-0.01em',
              marginBottom: 4,
            }}
          >
            审查意见分析
          </h2>
          <p style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
            共 {data.objections.length} 条审查意见，已回复 {data.responses.length} 条
            {isTauriRuntime() && reviewDoc ? ' · 数据来自案件 review 文档' : ''}
          </p>
        </div>
        <button
          type="button"
          onClick={() => void runKnowledgeHint()}
          disabled={knowledgeLoading || data.objections.length === 0}
          className="flex items-center"
          style={{
            gap: 6,
            padding: '6px 10px',
            fontSize: 11,
            fontWeight: 500,
            borderRadius: 6,
            border: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-elevated)',
            color: 'var(--accent-primary)',
          }}
        >
          {knowledgeLoading ? <Loader2 size={12} className="animate-spin" /> : <BookOpen size={12} />}
          知识库参考
        </button>
      </div>

      {knowledgeNote ? (
        <pre
          className="mb-4 custom-scrollbar"
          style={{
            maxHeight: 120,
            overflow: 'auto',
            fontSize: 11,
            padding: 10,
            borderRadius: 8,
            backgroundColor: 'var(--bg-elevated)',
            border: '1px solid var(--border-primary)',
            whiteSpace: 'pre-wrap',
          }}
        >
          {knowledgeNote}
        </pre>
      ) : null}

      {data.objections.length === 0 ? (
        <p style={{ fontSize: 13, color: 'var(--text-tertiary)' }}>
          当前案件无审查意见文档。选择 case-1 或刷新案件列表后将自动补全演示数据。
        </p>
      ) : null}

      <div className="flex flex-col" style={{ gap: 12 }}>
        {data.objections.map((obj, idx) => {
          const isExpanded = expandedObjections.has(obj.id);
          const response = data.responses.find((r) => r.objectionId === obj.id);

          return (
            <motion.div
              key={obj.id}
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: idx * 0.08, duration: 0.3 }}
              style={{
                backgroundColor: 'var(--bg-elevated)',
                borderRadius: 12,
                border: '1px solid var(--border-primary)',
                overflow: 'hidden',
              }}
            >
              <button
                onClick={() => toggleObjection(obj.id)}
                className="flex w-full items-center"
                style={{
                  padding: '12px 16px',
                  gap: 10,
                  backgroundColor: isExpanded ? 'var(--bg-sidebar-active)' : 'transparent',
                }}
                type="button"
              >
                {getTypeIcon(obj.type)}
                <div className="flex flex-1 flex-col items-start">
                  <div className="flex items-center" style={{ gap: 8 }}>
                    <span style={{ fontSize: 12, fontWeight: 600, color: getTypeColor(obj.type) }}>
                      {getTypeLabel(obj.type)}
                    </span>
                    <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>{obj.claim}</span>
                  </div>
                  {obj.citation ? (
                    <span style={{ fontSize: 10, color: 'var(--text-tertiary)', marginTop: 2 }}>
                      引用: {obj.citation}
                    </span>
                  ) : null}
                </div>
                <ChevronDown
                  size={14}
                  style={{
                    color: 'var(--text-tertiary)',
                    transform: isExpanded ? 'rotate(180deg)' : 'rotate(0deg)',
                    transition: 'transform 0.2s ease',
                  }}
                />
              </button>

              <AnimatePresence>
                {isExpanded && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="overflow-hidden"
                  >
                    <div style={{ padding: '12px 16px', borderTop: '1px solid var(--border-secondary)' }}>
                      <p style={{ fontSize: 13, lineHeight: 1.6, color: 'var(--text-primary)', marginBottom: 12 }}>
                        {obj.content}
                      </p>

                      <div
                        style={{
                          backgroundColor: 'var(--accent-primary-muted)',
                          borderRadius: 8,
                          padding: '10px 12px',
                        }}
                      >
                        <div className="mb-2 flex items-center" style={{ gap: 6 }}>
                          <CheckCircle size={12} style={{ color: 'var(--accent-primary)' }} />
                          <span style={{ fontSize: 11, fontWeight: 600, color: 'var(--accent-primary)' }}>
                            答复意见（自动保存）
                          </span>
                        </div>
                        <textarea
                          value={response?.content ?? ''}
                          onChange={(e) => updateResponse(obj.id, e.target.value)}
                          placeholder="在此撰写答复…"
                          className="w-full resize-y bg-transparent focus:outline-none"
                          style={{
                            minHeight: 72,
                            fontSize: 12,
                            lineHeight: 1.6,
                            color: 'var(--text-primary)',
                          }}
                        />
                      </div>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
          );
        })}
      </div>
    </motion.div>
  );
};

export default ReviewView;
