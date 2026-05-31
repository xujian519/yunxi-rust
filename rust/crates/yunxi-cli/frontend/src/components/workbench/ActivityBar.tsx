import type { FC, ComponentType, CSSProperties } from 'react';
import { useNavigate } from 'react-router';
import { FolderTree, GitCompare, ClipboardList, Search, PanelBottom, Settings } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import type { ViewType } from '@/data/mockData';
import { viewLabels } from '@/data/mockData';

interface ActivityItem {
  id: string;
  view?: ViewType;
  label: string;
  Icon: ComponentType<{ size?: number; style?: CSSProperties }>;
  /** 仅展开侧栏，不打开编辑器 */
  sidebarOnly?: boolean;
}

const items: ActivityItem[] = [
  { id: 'explorer', label: '资源管理器', Icon: FolderTree, sidebarOnly: true },
  { id: 'compare', view: 'compare', label: viewLabels.compare, Icon: GitCompare },
  { id: 'review', view: 'review', label: viewLabels.review, Icon: ClipboardList },
  { id: 'search', view: 'search', label: viewLabels.search, Icon: Search },
];

interface ActivityBarProps {
  onShowExplorer?: () => void;
}

const ActivityBar: FC<ActivityBarProps> = ({ onShowExplorer }) => {
  const navigate = useNavigate();
  const {
    sidebarActivity,
    setSidebarActivity,
    openToolView,
    activeView,
    bottomPanelVisible,
    toggleBottomPanel,
    toggleCommandPalette,
  } = useApp();

  return (
    <div
      className="flex h-full flex-col items-center justify-between"
      style={{
        width: 48,
        flexShrink: 0,
        backgroundColor: 'var(--bg-elevated)',
        borderRight: '1px solid var(--border-primary)',
        paddingTop: 8,
        paddingBottom: 8,
      }}
    >
      <div className="flex flex-col items-center" style={{ gap: 4 }}>
      {items.map(({ id, view, label, Icon, sidebarOnly }) => {
        const isExplorer = id === 'explorer';
        const isActive = isExplorer
          ? sidebarActivity === 'explorer'
          : view !== undefined && activeView === view;

        return (
          <button
            key={id}
            type="button"
            title={label}
            aria-label={label}
            onClick={() => {
              if (sidebarOnly) {
                setSidebarActivity('explorer');
                onShowExplorer?.();
                return;
              }
              if (view) openToolView(view);
            }}
            className="relative flex items-center justify-center transition-colors duration-150"
            style={{
              width: 40,
              height: 40,
              borderRadius: 8,
              color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
              backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
            }}
            onMouseEnter={(e) => {
              if (!isActive) {
                e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-hover, var(--bg-sidebar-active))';
                e.currentTarget.style.color = 'var(--text-secondary)';
              }
            }}
            onMouseLeave={(e) => {
              if (!isActive) {
                e.currentTarget.style.backgroundColor = 'transparent';
                e.currentTarget.style.color = 'var(--text-tertiary)';
              }
            }}
          >
            {isActive && (
              <span
                className="absolute left-0 rounded-r"
                style={{
                  width: 3,
                  height: 24,
                  backgroundColor: 'var(--accent-primary)',
                }}
              />
            )}
            <Icon size={22} />
          </button>
        );
      })}
      </div>
      <div className="flex flex-col items-center" style={{ gap: 4 }}>
        <button
          type="button"
          title="命令面板 (⇧⌘P)"
          aria-label="命令面板"
          onClick={() => toggleCommandPalette()}
          className="flex items-center justify-center"
          style={{
            width: 40,
            height: 40,
            borderRadius: 8,
            color: 'var(--text-tertiary)',
          }}
        >
          <Search size={20} />
        </button>
        <button
          type="button"
          title="设置"
          aria-label="设置"
          onClick={() => navigate('/settings')}
          className="flex items-center justify-center"
          style={{
            width: 40,
            height: 40,
            borderRadius: 8,
            color: 'var(--text-tertiary)',
          }}
        >
          <Settings size={20} />
        </button>
        <button
          type="button"
          title="底部面板"
          aria-label="底部面板"
          onClick={() => toggleBottomPanel('terminal')}
          className="relative flex items-center justify-center"
          style={{
            width: 40,
            height: 40,
            borderRadius: 8,
            color: bottomPanelVisible ? 'var(--text-primary)' : 'var(--text-tertiary)',
            backgroundColor: bottomPanelVisible ? 'var(--bg-sidebar-active)' : 'transparent',
          }}
        >
          {bottomPanelVisible && (
            <span
              className="absolute left-0 rounded-r"
              style={{ width: 3, height: 24, backgroundColor: 'var(--accent-primary)' }}
            />
          )}
          <PanelBottom size={22} />
        </button>
      </div>
    </div>
  );
};

export default ActivityBar;
