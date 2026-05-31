import { useEffect, useMemo, useState, useCallback } from 'react';
import type { FC } from 'react';
import { useNavigate } from 'react-router';
import {
  Search,
  Settings,
  FolderTree,
  GitCompare,
  ClipboardList,
  Terminal,
  PanelRight,
  RefreshCw,
  MessageSquarePlus,
  FileText,
  Sun,
  Download,
  FolderPlus,
  HelpCircle,
  BarChart3,
  BookOpen,
  ArrowLeft,
} from 'lucide-react';
import { Dialog, DialogContent } from '@/components/ui/dialog';
import { useApp } from '@/context/AppProvider';
import { useTheme } from '@/context/ThemeProvider';
import { viewLabels } from '@/data/mockData';
import type { ViewType } from '@/data/mockData';
import { isTauriRuntime } from '@/api';
import { filterPaletteCommands, type PaletteCommand } from '@/utils/commandPalette';
import { getPaletteRecent, recordPaletteRecent } from '@/utils/paletteRecent';

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onToggleSidebar?: () => void;
  onToggleAiPanel?: () => void;
}

interface PendingInput {
  label: string;
  slashTemplate: string;
}

const CommandPalette: FC<CommandPaletteProps> = ({
  open,
  onOpenChange,
  onToggleSidebar,
  onToggleAiPanel,
}) => {
  const navigate = useNavigate();
  const { toggle } = useTheme();
  const {
    openToolView,
    createSession,
    toggleBottomPanel,
    refreshWorkspaceScan,
    pickWorkspaceFolderDialog,
    executeSlashCommand,
    createCase,
    refreshCases,
    startImportProjectMaterials,
    workspaceProjects,
    openWorkspaceProject,
    activeCaseId,
    activeCase,
  } = useApp();

  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState(0);
  const [pendingInput, setPendingInput] = useState<PendingInput | null>(null);
  const [recentVersion, setRecentVersion] = useState(0);

  const close = useCallback(() => {
    onOpenChange(false);
    setQuery('');
    setPendingInput(null);
  }, [onOpenChange]);

  const run = useCallback(
    (cmd: PaletteCommand) => {
      if (cmd.needsInput && cmd.slashTemplate) {
        setPendingInput({ label: cmd.label, slashTemplate: cmd.slashTemplate });
        setQuery('');
        setSelected(0);
        return;
      }
      recordPaletteRecent(cmd.id);
      setRecentVersion((v) => v + 1);
      close();
      cmd.run();
    },
    [close],
  );

  const commands = useMemo((): PaletteCommand[] => {
    const openView = (view: ViewType, label: string): PaletteCommand => ({
      id: `view-${view}`,
      label: `打开：${label}`,
      group: '视图',
      keywords: [view, label],
      run: () => openToolView(view),
    });

    const list: PaletteCommand[] = [
      openView('search', viewLabels.search),
      openView('compare', viewLabels.compare),
      openView('review', viewLabels.review),
      openView('claims', viewLabels.claims),
      openView('draft', viewLabels.draft),
      {
        id: 'slash-search',
        label: '专利检索（输入关键词）',
        detail: '/search',
        group: '命令',
        keywords: ['检索', 'search', '专利'],
        needsInput: true,
        slashTemplate: '/search ',
        run: () => {},
      },
      {
        id: 'slash-analyze',
        label: '知识库分析（输入问题）',
        detail: '/analyze',
        group: '命令',
        keywords: ['分析', 'analyze', '知识库'],
        needsInput: true,
        slashTemplate: '/analyze ',
        run: () => {},
      },
      {
        id: 'slash-help',
        label: '显示 Slash 命令帮助',
        detail: '/help',
        group: '命令',
        run: () => void executeSlashCommand('/help'),
      },
      {
        id: 'slash-status',
        label: '显示会话与模型状态',
        detail: '/status',
        group: '命令',
        run: () => void executeSlashCommand('/status'),
      },
      {
        id: 'slash-cost',
        label: '刷新并显示费用',
        detail: '/cost',
        group: '命令',
        run: () => void executeSlashCommand('/cost'),
      },
      {
        id: 'new-session',
        label: '新建对话会话',
        group: '会话',
        run: () => void createSession(),
      },
      {
        id: 'new-case',
        label: '新建专利案件',
        group: '案件',
        run: () => {
          const name = window.prompt('案件名称：', '新案件');
          if (name?.trim()) void createCase(name.trim());
        },
      },
      {
        id: 'refresh-cases',
        label: '刷新案件列表',
        group: '案件',
        run: () => void refreshCases(),
      },
      {
        id: 'import-materials',
        label: '导入当前案件项目材料',
        detail: activeCase?.name,
        group: '案件',
        keywords: ['导入', '材料', 'markitdown'],
        run: () => {
          if (!activeCaseId) return;
          const project = workspaceProjects.find((p) => p.caseId === activeCaseId);
          if (project) {
            void startImportProjectMaterials(activeCaseId, project.folderPath);
          } else {
            const folder = window.prompt('项目目录绝对路径：');
            if (folder?.trim()) void startImportProjectMaterials(activeCaseId, folder.trim());
          }
        },
      },
      {
        id: 'toggle-sidebar',
        label: '切换资源管理器侧栏',
        detail: '⌘B',
        group: '布局',
        run: () => onToggleSidebar?.(),
      },
      {
        id: 'toggle-ai',
        label: '切换 AI 助手面板',
        detail: '⌘J',
        group: '布局',
        run: () => onToggleAiPanel?.(),
      },
      {
        id: 'toggle-terminal',
        label: '切换底栏终端',
        detail: '⌘`',
        group: '布局',
        run: () => toggleBottomPanel('terminal'),
      },
      {
        id: 'toggle-output',
        label: '打开底栏输出',
        group: '布局',
        run: () => toggleBottomPanel('output'),
      },
      {
        id: 'refresh-workspace',
        label: '重新扫描工作区',
        group: '工作区',
        run: () => void refreshWorkspaceScan(),
      },
      {
        id: 'add-workspace-folder',
        label: '添加工作区文件夹',
        group: '工作区',
        run: () => void pickWorkspaceFolderDialog(),
      },
      {
        id: 'settings',
        label: '打开设置',
        detail: '⌘,',
        group: '应用',
        run: () => navigate('/settings'),
      },
      {
        id: 'toggle-theme',
        label: '切换浅色/深色主题',
        detail: '⌘/',
        group: '应用',
        run: () => toggle(),
      },
      {
        id: 'focus-case-search',
        label: '聚焦案件搜索',
        detail: '⌘K',
        group: '导航',
        run: () => {
          document.querySelector<HTMLInputElement>('[data-explorer-search]')?.focus();
        },
      },
    ];

    if (!isTauriRuntime()) {
      return list;
    }

    for (const project of workspaceProjects.slice(0, 12)) {
      list.push({
        id: `project-${project.folderPath}`,
        label: `打开项目：${project.label}`,
        detail: project.folderPath,
        group: '工作区项目',
        keywords: [project.label, project.caseId ?? ''],
        run: () => openWorkspaceProject(project),
      });
    }

    return list;
  }, [
    openToolView,
    executeSlashCommand,
    createSession,
    createCase,
    refreshCases,
    startImportProjectMaterials,
    activeCaseId,
    activeCase,
    workspaceProjects,
    openWorkspaceProject,
    onToggleSidebar,
    onToggleAiPanel,
    toggleBottomPanel,
    refreshWorkspaceScan,
    pickWorkspaceFolderDialog,
    navigate,
    toggle,
  ]);

  const filtered = useMemo(() => {
    const base = filterPaletteCommands(commands, query);
    if (query.trim()) return base;
    const recentIds = getPaletteRecent();
    if (recentIds.length === 0) return base;
    const recentSet = new Set(recentIds);
    const recent = recentIds
      .map((id) => base.find((c) => c.id === id))
      .filter((c): c is PaletteCommand => !!c);
    const rest = base.filter((c) => !recentSet.has(c.id));
    return [...recent, ...rest];
  }, [commands, query, recentVersion, open]);

  useEffect(() => {
    setSelected(0);
  }, [query, open, pendingInput]);

  useEffect(() => {
    if (!open) {
      setQuery('');
      setPendingInput(null);
    }
  }, [open]);

  const submitPending = useCallback(() => {
    if (!pendingInput || !query.trim()) return;
    const id = pendingInput.slashTemplate.startsWith('/search')
      ? 'slash-search'
      : 'slash-analyze';
    recordPaletteRecent(id);
    setRecentVersion((v) => v + 1);
    void executeSlashCommand(`${pendingInput.slashTemplate}${query.trim()}`);
    close();
  }, [pendingInput, query, executeSlashCommand, close]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      if (pendingInput) {
        e.preventDefault();
        setPendingInput(null);
        setQuery('');
        return;
      }
    }
    if (pendingInput) {
      if (e.key === 'Enter') {
        e.preventDefault();
        submitPending();
      }
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelected((i) => Math.min(i + 1, Math.max(0, filtered.length - 1)));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelected((i) => Math.max(i - 1, 0));
    } else if (e.key === 'Enter' && filtered[selected]) {
      e.preventDefault();
      run(filtered[selected]);
    }
  };

  const iconFor = (id: string) => {
    if (id.startsWith('slash-search')) return <Search size={16} />;
    if (id.startsWith('slash-analyze')) return <BookOpen size={16} />;
    if (id.startsWith('slash-help')) return <HelpCircle size={16} />;
    if (id.startsWith('slash-status')) return <BarChart3 size={16} />;
    if (id.startsWith('slash-cost')) return <BarChart3 size={16} />;
    if (id.startsWith('view-search')) return <Search size={16} />;
    if (id.startsWith('view-compare')) return <GitCompare size={16} />;
    if (id.startsWith('view-review')) return <ClipboardList size={16} />;
    if (id.startsWith('view-')) return <FileText size={16} />;
    if (id.startsWith('project-')) return <FolderTree size={16} />;
    if (id === 'settings') return <Settings size={16} />;
    if (id === 'import-materials') return <Download size={16} />;
    if (id === 'new-case' || id === 'refresh-cases') return <FolderPlus size={16} />;
    if (id === 'toggle-terminal' || id === 'toggle-output') return <Terminal size={16} />;
    if (id === 'toggle-ai') return <PanelRight size={16} />;
    if (id === 'toggle-sidebar') return <FolderTree size={16} />;
    if (id === 'refresh-workspace' || id === 'add-workspace-folder') return <RefreshCw size={16} />;
    if (id === 'new-session') return <MessageSquarePlus size={16} />;
    if (id === 'toggle-theme') return <Sun size={16} />;
    return <Search size={16} />;
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        showCloseButton={false}
        className="gap-0 overflow-hidden p-0 sm:max-w-lg"
        style={{
          backgroundColor: 'var(--bg-elevated)',
          border: '1px solid var(--border-primary)',
        }}
      >
        <div
          className="flex items-center gap-2 border-b"
          style={{
            borderColor: 'var(--border-primary)',
            padding: '10px 12px',
          }}
        >
          {pendingInput ? (
            <button
              type="button"
              onClick={() => {
                setPendingInput(null);
                setQuery('');
              }}
              style={{ color: 'var(--text-tertiary)', flexShrink: 0 }}
              title="返回"
            >
              <ArrowLeft size={16} />
            </button>
          ) : (
            <Search size={16} style={{ color: 'var(--text-tertiary)', flexShrink: 0 }} />
          )}
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder={
              pendingInput
                ? `${pendingInput.label}…`
                : '输入命令（视图、案件、检索、工作区）…'
            }
            className="min-w-0 flex-1 bg-transparent focus:outline-none"
            style={{ fontSize: 14, color: 'var(--text-primary)' }}
          />
          <kbd
            style={{
              fontSize: 10,
              padding: '2px 6px',
              borderRadius: 4,
              color: 'var(--text-tertiary)',
              border: '1px solid var(--border-primary)',
            }}
          >
            ESC
          </kbd>
        </div>
        {!pendingInput && (
          <ul className="custom-scrollbar max-h-80 overflow-y-auto py-1" role="listbox">
            {filtered.length === 0 ? (
              <li style={{ padding: '12px 14px', fontSize: 12, color: 'var(--text-tertiary)' }}>
                无匹配命令
              </li>
            ) : (
              filtered.map((cmd, idx) => {
                const active = idx === selected;
                return (
                  <li key={cmd.id}>
                    <button
                      type="button"
                      role="option"
                      aria-selected={active}
                      onClick={() => run(cmd)}
                      onMouseEnter={() => setSelected(idx)}
                      className="flex w-full items-center gap-3 text-left"
                      style={{
                        padding: '8px 12px',
                        backgroundColor: active ? 'var(--accent-primary-muted)' : 'transparent',
                        color: 'var(--text-primary)',
                      }}
                    >
                      <span style={{ color: 'var(--text-tertiary)' }}>{iconFor(cmd.id)}</span>
                      <span className="min-w-0 flex-1">
                        <span style={{ display: 'block', fontSize: 13 }}>{cmd.label}</span>
                        {cmd.detail ? (
                          <span
                            className="block truncate"
                            style={{ fontSize: 11, color: 'var(--text-tertiary)' }}
                          >
                            {cmd.detail}
                          </span>
                        ) : null}
                      </span>
                      <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>{cmd.group}</span>
                    </button>
                  </li>
                );
              })
            )}
          </ul>
        )}
        {pendingInput && (
          <p style={{ padding: '10px 14px', fontSize: 12, color: 'var(--text-secondary)' }}>
            回车执行 {pendingInput.slashTemplate}
            <span style={{ color: 'var(--text-tertiary)' }}>关键词</span>
            ，结果写入 AI 对话与底栏「输出」
          </p>
        )}
      </DialogContent>
    </Dialog>
  );
};

export default CommandPalette;
