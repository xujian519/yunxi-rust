import type { FC } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import ClaimsView from './ClaimsView';
import CompareView from './CompareView';
import ReviewView from './ReviewView';
import SearchView from './SearchView';
import DraftView from './DraftView';
import ChatView from './ChatView';
import { viewLabels } from '@/data/mockData';
import type { ViewType } from '@/data/mockData';

interface CenterPanelProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
}

const tabViews: ViewType[] = ['claims', 'compare', 'review', 'search', 'draft', 'chat'];

const tabBadge: Record<ViewType, string | undefined> = {
  claims: '5',
  compare: undefined,
  review: '3',
  search: '5',
  draft: undefined,
  chat: undefined,
};

const CenterPanel: FC<CenterPanelProps> = ({ activeView, onViewChange }) => {
  const renderView = () => {
    switch (activeView) {
      case 'claims':
        return <ClaimsView />;
      case 'compare':
        return <CompareView />;
      case 'review':
        return <ReviewView />;
      case 'search':
        return <SearchView />;
      case 'draft':
        return <DraftView />;
      case 'chat':
        return <ChatView />;
      default:
        return <ClaimsView />;
    }
  };

  return (
    <div
      className="flex h-full flex-col overflow-hidden"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      {/* View Tab Bar */}
      <div
        className="flex items-center"
        style={{
          height: 40,
          padding: '0 8px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
          gap: 2,
          overflowX: 'auto',
        }}
      >
        {tabViews.map((view) => {
          const isActive = activeView === view;
          const badge = tabBadge[view];

          return (
            <button
              key={view}
              onClick={() => onViewChange(view)}
              className="relative flex items-center transition-colors duration-150"
              style={{
                height: 32,
                padding: '0 12px',
                borderRadius: 6,
                gap: 6,
                backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
                color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)',
                fontSize: 12,
                fontWeight: isActive ? 500 : 400,
                whiteSpace: 'nowrap',
                flexShrink: 0,
              }}
              onMouseEnter={(e) => {
                if (!isActive) e.currentTarget.style.color = 'var(--text-primary)';
              }}
              onMouseLeave={(e) => {
                if (!isActive) e.currentTarget.style.color = 'var(--text-secondary)';
              }}
              type="button"
            >
              {viewLabels[view]}
              {badge && (
                <span
                  style={{
                    fontSize: 10,
                    fontWeight: 500,
                    padding: '1px 5px',
                    borderRadius: 9999,
                    backgroundColor: 'var(--bg-sidebar-active)',
                    color: 'var(--text-tertiary)',
                  }}
                >
                  {badge}
                </span>
              )}
              {/* Active Indicator */}
              {isActive && (
                <motion.div
                  layoutId="active-tab-indicator"
                  className="absolute bottom-0 left-2 right-2"
                  style={{
                    height: 2,
                    borderRadius: 1,
                    backgroundColor: 'var(--accent-primary)',
                  }}
                  transition={{
                    type: 'spring',
                    stiffness: 400,
                    damping: 30,
                  }}
                />
              )}
            </button>
          );
        })}
      </div>

      {/* Content Area */}
      <div className="relative flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          <motion.div
            key={activeView}
            initial={{ opacity: 0, x: 16 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0 }}
            transition={{
              duration: 0.25,
              ease: [0.4, 0, 0.2, 1] as [number, number, number, number],
            }}
            className="h-full"
          >
            {renderView()}
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  );
};

export default CenterPanel;
