import type { FC, ComponentType, CSSProperties } from 'react';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ChevronLeft,
  ChevronRight,
  ChevronDown,
  FolderOpen,
  MessageSquare,
  FileText,
  GitCompare,
  Search,
  Globe,
  Edit3,
  MessageCircle,
  Plus,
  HelpCircle,
  Settings,
  Search as SearchIcon,
  X,
  Loader2,
} from 'lucide-react';
import { viewLabels } from '@/data/mockData';
import type { ViewType } from '@/data/mockData';
import { useApp } from '@/context/AppProvider';

const viewIcons: Record<ViewType, ComponentType<{ size?: number; style?: CSSProperties }>> = {
  claims: FileText,
  compare: GitCompare,
  review: Search,
  search: Globe,
  draft: Edit3,
  chat: MessageCircle,
};

interface LeftSidebarProps {
  isExpanded: boolean;
  onToggleExpand: () => void;
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
  width: number;
}

const LeftSidebar: FC<LeftSidebarProps> = ({
  isExpanded,
  onToggleExpand,
  activeView,
  onViewChange,
}) => {
  const navigate = useNavigate();
  const {
    cases,
    casesLoading,
    initError,
    activeCaseId,
    activeDocId,
    selectCase,
    selectDocument,
    createCase,
    sessions,
    activeSessionId,
    selectSession,
  } = useApp();
  const [expandedCases, setExpandedCases] = useState<Set<string>>(new Set(['case-1']));
  const [searchQuery, setSearchQuery] = useState('');
  const [hoveredSession, setHoveredSession] = useState<string | null>(null);

  const toggleCase = (id: string) => {
    setExpandedCases((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const caseItemColors = [
    'var(--accent-primary)',
    'var(--accent-secondary)',
    'var(--accent-cyan)',
  ];

  const views: ViewType[] = ['claims', 'compare', 'review', 'search', 'draft', 'chat'];

  const patentCasesFiltered = cases.filter((c) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return c.name.toLowerCase().includes(q) || c.number.toLowerCase().includes(q);
  });

  return (
    <div
      className="flex h-full flex-col"
      style={{
        backgroundColor: 'var(--bg-sidebar)',
        backdropFilter: 'blur(16px)',
        borderRight: '1px solid var(--border-primary)',
      }}
    >
      {/* Toggle Button */}
      <div className="flex items-center justify-center" style={{ padding: '8px 0 4px' }}>
        <button
          onClick={onToggleExpand}
          className="flex items-center justify-center transition-colors duration-150"
          style={{
            width: 28,
            height: 28,
            borderRadius: 6,
            color: 'var(--text-tertiary)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          type="button"
          aria-label={isExpanded ? '折叠侧边栏' : '展开侧边栏'}
        >
          {isExpanded ? <ChevronLeft size={16} /> : <ChevronRight size={16} />}
        </button>
      </div>

      {/* Search Input (expanded only) */}
      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.2 }}
            style={{ padding: '4px 12px 8px' }}
          >
            <div className="relative">
              <SearchIcon
                size={14}
                className="pointer-events-none absolute"
                style={{ left: 10, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-tertiary)' }}
              />
              <input
                type="text"
                placeholder="搜索专利..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full transition-all duration-200 focus:outline-none"
                style={{
                  height: 30,
                  padding: '6px 10px 6px 32px',
                  fontSize: 12,
                  borderRadius: 6,
                  backgroundColor: 'var(--bg-elevated)',
                  border: '1px solid var(--border-primary)',
                  color: 'var(--text-primary)',
                }}
                onFocus={(e) => {
                  e.currentTarget.style.borderColor = 'var(--border-focus)';
                  e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
                }}
                onBlur={(e) => {
                  e.currentTarget.style.borderColor = 'var(--border-primary)';
                  e.currentTarget.style.boxShadow = 'none';
                }}
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  className="absolute"
                  style={{ right: 8, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-tertiary)' }}
                  type="button"
                >
                  <X size={12} />
                </button>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* View Switcher */}
      <div
        className="flex items-center"
        style={{
          padding: isExpanded ? '4px 8px' : '4px 6px',
          gap: 2,
          flexWrap: isExpanded ? 'nowrap' : 'wrap',
          justifyContent: 'center',
        }}
      >
        {views.map((view) => {
          const Icon = viewIcons[view];
          const isActive = activeView === view;
          return (
            <button
              key={view}
              onClick={() => onViewChange(view)}
              className="relative flex items-center justify-center transition-all duration-200"
              style={{
                width: isExpanded ? 'auto' : 32,
                height: 32,
                padding: isExpanded ? '0 10px' : '0',
                borderRadius: 6,
                gap: isExpanded ? 6 : 0,
                backgroundColor: isActive ? 'var(--accent-primary-muted)' : 'transparent',
                color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)',
              }}
              onMouseEnter={(e) => {
                if (!isActive) {
                  e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
                  e.currentTarget.style.color = 'var(--text-secondary)';
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive) {
                  e.currentTarget.style.backgroundColor = 'transparent';
                  e.currentTarget.style.color = 'var(--text-tertiary)';
                }
              }}
              title={!isExpanded ? viewLabels[view] : undefined}
              type="button"
            >
              <Icon size={16} />
              <AnimatePresence>
                {isExpanded && (
                  <motion.span
                    initial={{ opacity: 0, width: 0 }}
                    animate={{ opacity: 1, width: 'auto' }}
                    exit={{ opacity: 0, width: 0 }}
                    transition={{ duration: 0.2, delay: 0.05 }}
                    className="overflow-hidden whitespace-nowrap"
                    style={{ fontSize: 11, fontWeight: 500 }}
                  >
                    {viewLabels[view]}
                  </motion.span>
                )}
              </AnimatePresence>
            </button>
          );
        })}
      </div>

      {/* Scrollable Content */}
      <div
        className="flex-1 overflow-y-auto custom-scrollbar"
        style={{ padding: '8px 0' }}
      >
        {/* Patent Cases Section */}
        {isExpanded && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.1 }}
          >
            <div
              style={{
                padding: '4px 12px',
                fontSize: 10,
                fontWeight: 600,
                letterSpacing: '0.05em',
                textTransform: 'uppercase',
                color: 'var(--text-tertiary)',
              }}
            >
              案件
            </div>
          </motion.div>
        )}

        {casesLoading && isExpanded && (
          <div
            className="flex items-center"
            style={{ padding: '8px 12px', gap: 6, fontSize: 11, color: 'var(--text-tertiary)' }}
          >
            <Loader2 size={12} className="animate-spin" />
            加载案件…
          </div>
        )}

        {initError && isExpanded && (
          <div style={{ padding: '8px 12px', fontSize: 11, color: 'var(--status-error)' }}>
            {initError}
          </div>
        )}

        {patentCasesFiltered.map((caseItem, idx) => {
          const isExpandedCase = expandedCases.has(caseItem.id);
          const isActive = activeCaseId === caseItem.id;
          const caseColor = caseItemColors[idx % caseItemColors.length];

          return (
            <div key={caseItem.id}>
              <button
                onClick={() => {
                  selectCase(caseItem.id);
                  toggleCase(caseItem.id);
                }}
                className="flex w-full items-center transition-colors duration-150"
                style={{
                  height: 32,
                  padding: isExpanded ? '6px 12px' : '6px 0',
                  justifyContent: isExpanded ? 'flex-start' : 'center',
                  backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
                }}
                onMouseEnter={(e) => {
                  if (!isActive) e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
                }}
                onMouseLeave={(e) => {
                  if (!isActive) e.currentTarget.style.backgroundColor = 'transparent';
                }}
                type="button"
              >
                {isExpanded && (
                  <ChevronDown
                    size={12}
                    className="transition-transform duration-200"
                    style={{
                      marginRight: 4,
                      color: 'var(--text-tertiary)',
                      transform: isExpandedCase ? 'rotate(0deg)' : 'rotate(-90deg)',
                    }}
                  />
                )}
                <FolderOpen
                  size={16}
                  style={{
                    color: isActive ? caseColor : 'var(--text-tertiary)',
                    marginRight: isExpanded ? 6 : 0,
                    flexShrink: 0,
                  }}
                />
                <AnimatePresence>
                  {isExpanded && (
                    <motion.div
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      transition={{ duration: 0.2, delay: 0.05 }}
                      className="flex min-w-0 flex-1 flex-col items-start overflow-hidden"
                    >
                      <span
                        className="block truncate"
                        style={{ fontSize: 12, fontWeight: 500, color: 'var(--text-primary)' }}
                      >
                        {caseItem.name}
                      </span>
                      <span
                        className="block truncate"
                        style={{ fontSize: 10, color: 'var(--text-tertiary)' }}
                      >
                        {caseItem.number}
                      </span>
                    </motion.div>
                  )}
                </AnimatePresence>
              </button>

              {/* Child items */}
              <AnimatePresence>
                {isExpanded && isExpandedCase && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    exit={{ opacity: 0, height: 0 }}
                    transition={{ duration: 0.2 }}
                    className="overflow-hidden"
                  >
                    {caseItem.children.map((child, childIdx) => {
                      const isChildActive = activeDocId === child.id;
                      return (
                        <motion.button
                          key={child.id}
                          initial={{ opacity: 0, x: -4 }}
                          animate={{ opacity: 1, x: 0 }}
                          transition={{ delay: childIdx * 0.03 }}
                          onClick={() => {
                            selectDocument(caseItem.id, child.id, child.type);
                          }}
                          className="flex w-full items-center transition-colors duration-150"
                          style={{
                            height: 28,
                            padding: '4px 12px 4px 36px',
                            backgroundColor: isChildActive
                              ? 'var(--bg-sidebar-active)'
                              : 'transparent',
                            borderLeft: isChildActive
                              ? '2px solid var(--accent-primary)'
                              : '2px solid transparent',
                          }}
                          onMouseEnter={(e) => {
                            if (!isChildActive)
                              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
                          }}
                          onMouseLeave={(e) => {
                            if (!isChildActive)
                              e.currentTarget.style.backgroundColor = 'transparent';
                          }}
                          type="button"
                        >
                          <FileText
                            size={13}
                            style={{
                              color: isChildActive
                                ? 'var(--accent-primary)'
                                : 'var(--text-tertiary)',
                              marginRight: 6,
                              flexShrink: 0,
                            }}
                          />
                          <span
                            className="truncate"
                            style={{
                              fontSize: 11,
                              color: isChildActive
                                ? 'var(--text-primary)'
                                : 'var(--text-secondary)',
                            }}
                          >
                            {child.name}
                          </span>
                        </motion.button>
                      );
                    })}
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          );
        })}

        {/* Sessions Section */}
        <div style={{ marginTop: 12 }}>
          {isExpanded && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.1 }}
            >
              <div
                style={{
                  padding: '4px 12px',
                  fontSize: 10,
                  fontWeight: 600,
                  letterSpacing: '0.05em',
                  textTransform: 'uppercase',
                  color: 'var(--text-tertiary)',
                }}
              >
                会话
              </div>
            </motion.div>
          )}

          {sessions.map((session) => (
            <button
              key={session.id}
              onClick={() => void selectSession(session.id)}
              className="flex w-full items-center transition-colors duration-150"
              style={{
                height: 30,
                padding: isExpanded ? '4px 12px' : '4px 0',
                justifyContent: isExpanded ? 'flex-start' : 'center',
                backgroundColor:
                  activeSessionId === session.id ? 'var(--bg-sidebar-active)' : 'transparent',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
                setHoveredSession(session.id);
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
                setHoveredSession(null);
              }}
              type="button"
            >
              <MessageSquare
                size={14}
                style={{
                  color: 'var(--text-tertiary)',
                  marginRight: isExpanded ? 6 : 0,
                  flexShrink: 0,
                }}
              />
              <AnimatePresence>
                {isExpanded && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="flex min-w-0 flex-1 items-center justify-between"
                  >
                    <span
                      className="truncate"
                      style={{ fontSize: 11, color: 'var(--text-secondary)' }}
                    >
                      {session.title}
                    </span>
                    <span className="flex items-center" style={{ gap: 4 }}>
                      <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>
                        {session.timestamp}
                      </span>
                      <AnimatePresence>
                        {hoveredSession === session.id && (
                          <motion.span
                            initial={{ opacity: 0, scale: 0.8 }}
                            animate={{ opacity: 1, scale: 1 }}
                            exit={{ opacity: 0, scale: 0.8 }}
                            transition={{ duration: 0.1 }}
                            style={{ color: 'var(--text-tertiary)', cursor: 'pointer' }}
                            onClick={(e) => e.stopPropagation()}
                          >
                            <X size={12} />
                          </motion.span>
                        )}
                      </AnimatePresence>
                    </span>
                  </motion.div>
                )}
              </AnimatePresence>
            </button>
          ))}
        </div>
      </div>

      {/* Bottom Actions */}
      <div
        className="flex items-center"
        style={{
          padding: isExpanded ? '8px 12px' : '8px 0',
          borderTop: '1px solid var(--border-primary)',
          justifyContent: isExpanded ? 'space-between' : 'center',
          gap: 4,
        }}
      >
        <button
          onClick={() => void createCase('新案件')}
          className="flex items-center justify-center transition-all duration-200"
          style={{
            width: isExpanded ? 'auto' : 32,
            height: 32,
            padding: isExpanded ? '0 12px' : '0',
            borderRadius: 9999,
            backgroundColor: 'var(--accent-primary)',
            color: 'var(--text-inverse)',
            gap: isExpanded ? 6 : 0,
            fontSize: 11,
            fontWeight: 500,
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'scale(1.05)';
            e.currentTarget.style.boxShadow = '0 4px 12px rgba(74, 124, 111, 0.3)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'scale(1)';
            e.currentTarget.style.boxShadow = 'none';
          }}
          type="button"
        >
          <Plus size={16} />
          <AnimatePresence>
            {isExpanded && (
              <motion.span
                initial={{ opacity: 0, width: 0 }}
                animate={{ opacity: 1, width: 'auto' }}
                exit={{ opacity: 0, width: 0 }}
                transition={{ duration: 0.2 }}
                className="overflow-hidden whitespace-nowrap"
              >
                新建案件
              </motion.span>
            )}
          </AnimatePresence>
        </button>
        <div className="flex items-center" style={{ gap: 4 }}>
          <button
            className="flex items-center justify-center transition-colors duration-150"
            style={{
              width: 28,
              height: 28,
              borderRadius: 6,
              color: 'var(--text-tertiary)',
            }}
            onClick={() => navigate('/settings')}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
              e.currentTarget.style.color = 'var(--text-secondary)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
              e.currentTarget.style.color = 'var(--text-tertiary)';
            }}
            type="button"
            aria-label="设置"
            title="设置"
          >
            <Settings size={16} />
          </button>
          <button
            className="flex items-center justify-center transition-colors duration-150"
            style={{
              width: 28,
              height: 28,
              borderRadius: 6,
              color: 'var(--text-tertiary)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
              e.currentTarget.style.color = 'var(--text-secondary)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
              e.currentTarget.style.color = 'var(--text-tertiary)';
            }}
            type="button"
            aria-label="帮助"
            title="帮助"
          >
            <HelpCircle size={16} />
          </button>
        </div>
      </div>
    </div>
  );
};

export default LeftSidebar;
