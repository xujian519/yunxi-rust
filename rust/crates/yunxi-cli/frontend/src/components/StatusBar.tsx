import type { FC } from 'react';
import { Sun, Moon, Wifi, WifiOff, GitBranch, Cpu, AlertCircle, PanelBottom } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import { useTheme } from '@/context/ThemeProvider';
import { isTauriRuntime } from '@/api';

const StatusBar: FC = () => {
  const {
    usage,
    model,
    ready,
    initError,
    budgetTotal,
    problemCount,
    toggleBottomPanel,
    activeWorkspaceFolder,
  } = useApp();
  const { resolved, toggle } = useTheme();
  const isDark = resolved === 'dark';

  const isOnline = ready && !initError;
  const costUsed = usage?.estimated_cost ?? 0;
  const costTotal = budgetTotal;

  const costPercent = costTotal > 0 ? Math.min((costUsed / costTotal) * 100, 100) : 0;
  const costColor =
    costPercent > 80
      ? 'var(--status-error)'
      : costPercent > 50
        ? 'var(--status-warning)'
        : 'var(--status-success)';

  const modelLabel = model.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());

  return (
    <footer
      className="flex items-center justify-between select-none shrink-0"
      style={{
        height: 28,
        backgroundColor: 'var(--bg-surface)',
        borderTop: '1px solid var(--border-primary)',
        padding: '0 12px',
        fontSize: 11,
        fontWeight: 500,
        letterSpacing: '0.01em',
        color: 'var(--text-tertiary)',
      }}
    >
      <div className="flex items-center" style={{ gap: 16 }}>
        <div className="flex items-center" style={{ gap: 4 }}>
          {isOnline ? (
            <Wifi size={12} style={{ color: 'var(--status-success)' }} />
          ) : (
            <WifiOff size={12} style={{ color: 'var(--status-error)' }} />
          )}
          <span>{initError ? '连接异常' : isOnline ? '已连接' : '初始化中…'}</span>
        </div>

        <div className="flex items-center" style={{ gap: 6 }}>
          <span>费用</span>
          <div
            style={{
              width: 60,
              height: 4,
              borderRadius: 2,
              backgroundColor: 'var(--border-primary)',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                width: `${costPercent}%`,
                height: '100%',
                backgroundColor: costColor,
                borderRadius: 2,
                transition: 'width 0.3s ease, background-color 0.3s ease',
              }}
            />
          </div>
          <span>
            ${costUsed.toFixed(2)} / ${costTotal.toFixed(2)}
          </span>
        </div>
      </div>

      <div className="flex items-center" style={{ gap: 12 }}>
        {problemCount > 0 && (
          <button
            type="button"
            onClick={() => toggleBottomPanel('problems')}
            className="flex items-center"
            style={{ gap: 4, color: 'var(--status-error)' }}
            title="查看问题"
          >
            <AlertCircle size={12} />
            <span>
              {problemCount} 个问题
            </span>
          </button>
        )}
        <button
          type="button"
          onClick={() => toggleBottomPanel('output')}
          className="flex items-center"
          style={{ gap: 4, color: 'var(--text-tertiary)' }}
          title="输出面板"
        >
          <PanelBottom size={12} />
          <span>面板</span>
        </button>
        <div className="flex items-center" style={{ gap: 4 }}>
          <GitBranch size={10} />
          <span className="max-w-[140px] truncate" title={activeWorkspaceFolder?.path}>
            {activeWorkspaceFolder?.label ?? (isTauriRuntime() ? 'desktop' : 'web-mock')}
          </span>
        </div>
        <div className="flex items-center" style={{ gap: 4 }}>
          <Cpu size={10} />
          <span>{modelLabel}</span>
        </div>
      </div>

      <div className="flex items-center" style={{ gap: 8 }}>
        <button
          onClick={toggle}
          className="flex items-center justify-center"
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            color: 'var(--text-tertiary)',
            transition: 'color 0.15s ease, background-color 0.15s ease',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.color = 'var(--text-secondary)';
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.color = 'var(--text-tertiary)';
            e.currentTarget.style.backgroundColor = 'transparent';
          }}
          title={isDark ? '切换到浅色模式' : '切换到深色模式'}
          type="button"
        >
          {isDark ? <Sun size={12} /> : <Moon size={12} />}
        </button>
      </div>
    </footer>
  );
};

export default StatusBar;
