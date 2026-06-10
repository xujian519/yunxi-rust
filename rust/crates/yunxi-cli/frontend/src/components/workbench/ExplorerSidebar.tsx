import type { FC } from 'react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import {
  ChevronDown,
  ChevronRight,
  FolderOpen,
  FolderPlus,
  MessageSquare,
  FileText,
  Plus,
  HelpCircle,
  Settings,
  Search as SearchIcon,
  X,
  Loader2,
  Trash2,
  Download,
  Eye,
  EyeOff,
} from 'lucide-react';
import type { DirectoryEntry } from '@/api/types';
import { api, isTauriRuntime } from '@/api';
import { useApp } from '@/context/AppProvider';

interface ExplorerSidebarProps {
  isExpanded: boolean;
  onToggleExpand: () => void;
}

const ExplorerSidebar: FC<ExplorerSidebarProps> = ({ isExpanded, onToggleExpand }) => {
  const navigate = useNavigate();
  const {
    cases,
    casesLoading,
    initError,
    activeCaseId,
    activeDocId,
    selectCase,
    openDocument,
    openExternalFile,
    createCase,
    deleteCase,
    sessions,
    activeSessionId,
    selectSession,
    createSession,
    deleteSession,
    visibleWorkspaceFolders,
    activeWorkspaceFolderId,
    setActiveWorkspaceFolder,
    pickWorkspaceFolderDialog,
    removeWorkspaceFolder,
    visibleWorkspaceProjects,
    workspaceWatchEnabled,
    setWorkspaceWatchEnabled,
    workspaceScanning,
    refreshWorkspaceScan,
    openWorkspaceProject,
    startImportProjectMaterials,
    workspaceScanMaxDepth,
    setWorkspaceScanMaxDepth,
  } = useApp();
  const [expandedCases, setExpandedCases] = useState<Set<string>>(new Set());
  const [hoveredCase, setHoveredCase] = useState<string | null>(null);
  const [workspaceExpanded, setWorkspaceExpanded] = useState(true);
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');
  const [hoveredSession, setHoveredSession] = useState<string | null>(null);
  const [dirCache, setDirCache] = useState<Record<string, DirectoryEntry[]>>({});
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());

  const loadDirectory = useCallback(async (dir: string) => {
    if (dirCache[dir] || !isTauriRuntime()) return;
    try {
      const entries = await api.listDirectory(dir);
      setDirCache((prev) => ({ ...prev, [dir]: entries }));
    } catch {
      // ignore
    }
  }, [dirCache]);

  const toggleCase = (id: string) => {
    setExpandedCases((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  useEffect(() => {
    if (activeCaseId) {
      setExpandedCases((prev) => new Set(prev).add(activeCaseId));
    }
  }, [activeCaseId]);

  const caseItemColors = [
    'var(--accent-primary)',
    'var(--accent-secondary)',
    'var(--accent-cyan)',
  ];

  const patentCasesFiltered = cases.filter((c) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return c.name.toLowerCase().includes(q) || c.number.toLowerCase().includes(q);
  });

  const toggleDir = (dir: string) => {
    setExpandedDirs((prev) => {
      const next = new Set(prev);
      if (next.has(dir)) next.delete(dir);
      else {
        next.add(dir);
        void loadDirectory(dir);
      }
      return next;
    });
  };

  const renderFileTree = (rootPath: string, baseIndent: number): React.ReactNode => {
    const entries = dirCache[rootPath];
    if (!entries) return null;
    return entries.map((entry) => {
      const isOpen = expandedDirs.has(entry.path);
      return (
        <div key={entry.path}>
          <div
            className="flex w-full items-center"
            style={{ height: 24, padding: `2px 12px 2px ${baseIndent + 16}px` }}
          >
            <button
              type="button"
              onClick={() => {
                if (entry.isDir) {
                  toggleDir(entry.path);
                } else {
                  openExternalFile(entry.path, entry.name);
                }
              }}
              className="flex min-w-0 flex-1 items-center truncate text-left"
              style={{ fontSize: 11, color: 'var(--text-secondary)' }}
            >
              {entry.isDir ? (
                <>
                  <ChevronRight
                    size={10}
                    style={{
                      marginRight: 2,
                      color: 'var(--text-tertiary)',
                      transform: isOpen ? 'rotate(90deg)' : 'rotate(0deg)',
                      flexShrink: 0,
                    }}
                  />
                  <FolderOpen
                    size={12}
                    style={{
                      marginRight: 4,
                      color: 'var(--text-tertiary)',
                      flexShrink: 0,
                    }}
                  />
                </>
              ) : (
                <FileText
                  size={12}
                  style={{
                    marginRight: 4,
                    marginLeft: 12,
                    color: 'var(--text-tertiary)',
                    flexShrink: 0,
                  }}
                />
              )}
              <span className="truncate">{entry.name}</span>
            </button>
          </div>
          {isOpen && entry.isDir && renderFileTree(entry.path, baseIndent + 12)}
        </div>
      );
    });
  };

  return (
    <div
      className="flex h-full flex-col"
      style={{
        backgroundColor: 'var(--bg-sidebar)',
        borderRight: '1px solid var(--border-primary)',
      }}
    >
      {/* 面板标题栏（VS Code 风格） */}
      <div
        className="flex items-center justify-between"
        style={{
          height: 35,
          padding: isExpanded ? '0 12px' : '0 4px',
          borderBottom: '1px solid var(--border-primary)',
          fontSize: 11,
          fontWeight: 600,
          letterSpacing: '0.04em',
          textTransform: 'uppercase',
          color: 'var(--text-secondary)',
        }}
      >
        {isExpanded ? <span>资源管理器</span> : null}
        <button
          type="button"
          onClick={onToggleExpand}
          aria-label={isExpanded ? '折叠侧栏' : '展开侧栏'}
          style={{
            fontSize: 10,
            color: 'var(--text-tertiary)',
            padding: '2px 6px',
          }}
        >
          {isExpanded ? '◂' : '▸'}
        </button>
      </div>

      {isExpanded && (
        <div style={{ padding: '6px 10px 8px' }}>
          <div className="relative">
            <SearchIcon
              size={14}
              className="pointer-events-none absolute"
              style={{
                left: 10,
                top: '50%',
                transform: 'translateY(-50%)',
                color: 'var(--text-tertiary)',
              }}
            />
            <input
              type="text"
              data-explorer-search
              placeholder="搜索案件…"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full focus:outline-none"
              style={{
                height: 28,
                padding: '4px 10px 4px 30px',
                fontSize: 12,
                borderRadius: 6,
                backgroundColor: 'var(--bg-elevated)',
                border: '1px solid var(--border-primary)',
                color: 'var(--text-primary)',
              }}
            />
            {searchQuery ? (
              <button
                type="button"
                onClick={() => setSearchQuery('')}
                className="absolute"
                style={{
                  right: 8,
                  top: '50%',
                  transform: 'translateY(-50%)',
                  color: 'var(--text-tertiary)',
                }}
              >
                <X size={12} />
              </button>
            ) : null}
          </div>
        </div>
      )}

      <div className="custom-scrollbar flex-1 overflow-y-auto" style={{ padding: '4px 0' }}>
        {isExpanded && (
          <>
            <div
              className="flex w-full items-center"
              style={{
                padding: '4px 12px',
                fontSize: 10,
                fontWeight: 600,
                letterSpacing: '0.05em',
                textTransform: 'uppercase',
                color: 'var(--text-tertiary)',
              }}
            >
              <button
                type="button"
                onClick={() => setWorkspaceExpanded((v) => !v)}
                className="flex flex-1 items-center"
              >
                <ChevronDown
                  size={12}
                  style={{
                    marginRight: 4,
                    transform: workspaceExpanded ? 'rotate(0deg)' : 'rotate(-90deg)',
                  }}
                />
                工作区
              </button>
              <select
                value={workspaceScanMaxDepth}
                onChange={(e) => {
                  setWorkspaceScanMaxDepth(Number(e.target.value));
                  void refreshWorkspaceScan();
                }}
                title="扫描深度"
                style={{
                  fontSize: 10,
                  marginRight: 4,
                  maxWidth: 36,
                  color: 'var(--text-tertiary)',
                  background: 'transparent',
                  border: 'none',
                }}
              >
                {[1, 2, 3, 4, 5].map((d) => (
                  <option key={d} value={d}>
                    {d}
                  </option>
                ))}
              </select>
              <button
                type="button"
                onClick={() => void refreshWorkspaceScan()}
                title="重新扫描 YUNXI.md"
                style={{ color: 'var(--text-tertiary)', marginRight: 4 }}
              >
                {workspaceScanning ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : (
                  <SearchIcon size={12} />
                )}
              </button>
              <button
                type="button"
                onClick={() => setWorkspaceWatchEnabled(!workspaceWatchEnabled)}
                title={
                  workspaceWatchEnabled
                    ? '关闭自动监视（减少刷新与输出跳动）'
                    : '开启自动监视（文件变更后自动扫描）'
                }
                style={{
                  color: workspaceWatchEnabled ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                  marginRight: 4,
                }}
              >
                {workspaceWatchEnabled ? <Eye size={12} /> : <EyeOff size={12} />}
              </button>
              <button
                type="button"
                onClick={() => void pickWorkspaceFolderDialog()}
                title="添加文件夹（原生选择器）"
                style={{ color: 'var(--text-tertiary)' }}
              >
                <FolderPlus size={12} />
              </button>
            </div>
            {workspaceExpanded &&
              visibleWorkspaceFolders.map((folder) => {
                const isActive = activeWorkspaceFolderId === folder.id;
                const projects = visibleWorkspaceProjects.filter(
                  (p) => p.workspaceRoot === folder.path,
                );
                const isFolderExpanded = expandedFolders.has(folder.id);
                return (
                  <div key={folder.id}>
                    <div
                      className="group flex w-full items-center"
                      style={{
                        height: 28,
                        padding: '4px 12px 4px 16px',
                        backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
                      }}
                    >
                      <button
                        type="button"
                        onClick={() => {
                          setActiveWorkspaceFolder(folder.id);
                          setExpandedFolders((prev) => {
                            const next = new Set(prev);
                            if (next.has(folder.id)) next.delete(folder.id);
                            else {
                              next.add(folder.id);
                              void loadDirectory(folder.path);
                            }
                            return next;
                          });
                        }}
                        className="flex min-w-0 flex-1 items-center truncate text-left"
                        title={folder.path}
                      >
                        <ChevronDown
                          size={12}
                          style={{
                            marginRight: 2,
                            color: 'var(--text-tertiary)',
                            transform: isFolderExpanded ? 'rotate(0deg)' : 'rotate(-90deg)',
                            flexShrink: 0,
                          }}
                        />
                        <FolderOpen
                          size={14}
                          style={{
                            marginRight: 6,
                            color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                          }}
                        />
                        <span
                          className="truncate"
                          style={{ fontSize: 11, color: 'var(--text-secondary)' }}
                        >
                          {folder.label}
                          {folder.isPrimary ? ' (主)' : ''}
                        </span>
                      </button>
                      <div className="flex shrink-0 items-center" style={{ gap: 4 }}>
                        {!folder.isPrimary && (
                          <button
                            type="button"
                            onClick={() => {
                              if (
                                !window.confirm(
                                  `确定从工作区移除「${folder.label}」？\n${folder.path}`,
                                )
                              ) {
                                return;
                              }
                              removeWorkspaceFolder(folder.id);
                            }}
                            title="从工作区删除"
                            style={{ color: 'var(--text-tertiary)' }}
                          >
                            <Trash2 size={12} />
                          </button>
                        )}
                      </div>
                    </div>
                    {isFolderExpanded &&
                      projects.map((project) => (
                      <div
                        key={project.folderPath}
                        className="group flex w-full items-center"
                        style={{
                          height: 26,
                          padding: '4px 12px 4px 44px',
                        }}
                      >
                        <button
                          type="button"
                          onClick={() => openWorkspaceProject(project)}
                          className="flex min-w-0 flex-1 items-center truncate text-left"
                          title={project.folderPath}
                          style={{ fontSize: 11, color: 'var(--text-secondary)' }}
                        >
                          <FileText
                            size={12}
                            style={{
                              marginRight: 6,
                              flexShrink: 0,
                              color: project.isPatentProject
                                ? 'var(--accent-primary)'
                                : 'var(--text-tertiary)',
                            }}
                          />
                          <span className="truncate">{project.label}</span>
                          {project.isPatentProject && (
                            <span
                              style={{
                                marginLeft: 4,
                                fontSize: 9,
                                color: 'var(--accent-cyan)',
                            }}
                          >
                            专利
                          </span>
                          )}
                        </button>
                        <div className="flex shrink-0 items-center" style={{ gap: 4 }}>
                          {project.caseId && (
                            <button
                              type="button"
                              title="从项目目录导入材料到关联案件"
                              style={{ color: 'var(--text-tertiary)', flexShrink: 0 }}
                              onClick={(e) => {
                                e.stopPropagation();
                                void startImportProjectMaterials(
                                  project.caseId!,
                                  project.folderPath,
                                );
                              }}
                            >
                              <Download size={12} />
                            </button>
                          )}
                        </div>
                      </div>
                    ))}
                    {isFolderExpanded && renderFileTree(folder.path, 44)}
                  </div>
                );
              })}
          </>
        )}

        {isExpanded && (
          <div
            style={{
              padding: '8px 12px 4px',
              fontSize: 10,
              fontWeight: 600,
              letterSpacing: '0.05em',
              textTransform: 'uppercase',
              color: 'var(--text-tertiary)',
            }}
          >
            案件
          </div>
        )}

        {isExpanded && (
          <div
            className="flex items-center"
            style={{
              minHeight: 28,
              padding: '8px 12px',
              gap: 6,
              fontSize: 11,
              color: 'var(--text-tertiary)',
              visibility: casesLoading ? 'visible' : 'hidden',
            }}
          >
            <Loader2
              size={12}
              className={casesLoading ? 'animate-spin' : ''}
              style={{ opacity: casesLoading ? 1 : 0 }}
            />
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
              <div
                className="flex w-full items-center"
                style={{
                  height: 32,
                  padding: isExpanded ? '0 4px 0 0' : '0',
                }}
                onMouseEnter={() => setHoveredCase(caseItem.id)}
                onMouseLeave={() => setHoveredCase(null)}
              >
                <button
                  type="button"
                  onClick={() => {
                    selectCase(caseItem.id);
                    toggleCase(caseItem.id);
                  }}
                  className="flex min-w-0 flex-1 items-center transition-colors duration-150"
                  style={{
                    height: 32,
                    padding: isExpanded ? '6px 8px 6px 12px' : '6px 0',
                    justifyContent: isExpanded ? 'flex-start' : 'center',
                    backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
                  }}
                >
                  {isExpanded && (
                    <ChevronDown
                      size={12}
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
                    }}
                  />
                  {isExpanded && (
                    <div className="min-w-0 flex-1 overflow-hidden text-left">
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
                    </div>
                  )}
                </button>
                {isExpanded && hoveredCase === caseItem.id && (
                  <div className="flex shrink-0 items-center" style={{ gap: 2, paddingRight: 4 }}>
                    <button
                      type="button"
                      title="删除案件"
                      onClick={(e) => {
                        e.stopPropagation();
                        if (
                          !window.confirm(
                            `确定删除案件「${caseItem.name}」？\n此操作不可恢复。`,
                          )
                        ) {
                          return;
                        }
                        void deleteCase(caseItem.id);
                      }}
                      style={{ color: 'var(--text-tertiary)', width: 22, height: 22 }}
                    >
                      <Trash2 size={12} />
                    </button>
                  </div>
                )}
              </div>

              {isExpanded && isExpandedCase && (
                <div>
                  {caseItem.children.map((child) => {
                    const isChildActive = activeDocId === child.id;
                    return (
                      <button
                        key={child.id}
                        type="button"
                        onClick={() =>
                          openDocument(caseItem.id, child.id, child.type, child.name)
                        }
                        className="flex w-full items-center"
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
                      >
                        <FileText
                          size={13}
                          style={{
                            color: isChildActive
                              ? 'var(--accent-primary)'
                              : 'var(--text-tertiary)',
                            marginRight: 6,
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
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}

        <div style={{ marginTop: 12 }}>
          {isExpanded && (
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
          )}

          {sessions.map((session) => (
            <div
              key={session.id}
              className="flex w-full items-center"
              style={{
                height: 30,
                padding: isExpanded ? '4px 12px' : '4px 0',
                justifyContent: isExpanded ? 'flex-start' : 'center',
                backgroundColor:
                  activeSessionId === session.id ? 'var(--bg-sidebar-active)' : 'transparent',
              }}
              onMouseEnter={() => setHoveredSession(session.id)}
              onMouseLeave={() => setHoveredSession(null)}
            >
              <button
                type="button"
                onClick={() => void selectSession(session.id)}
                className="flex min-w-0 flex-1 items-center truncate text-left"
              >
                <MessageSquare
                  size={14}
                  style={{ color: 'var(--text-tertiary)', marginRight: isExpanded ? 6 : 0 }}
                />
                {isExpanded && (
                  <span
                    className="truncate"
                    style={{ fontSize: 11, color: 'var(--text-secondary)', flex: 1 }}
                  >
                    {session.title}
                  </span>
                )}
              </button>
              {isExpanded && hoveredSession === session.id && (
                <div className="flex shrink-0 items-center" style={{ gap: 2 }}>
                  <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>
                    {session.timestamp}
                  </span>
                  <button
                    type="button"
                    title="删除会话"
                    onClick={(e) => {
                      e.stopPropagation();
                      if (!window.confirm(`确定删除会话「${session.title}」？`)) return;
                      void deleteSession(session.id);
                    }}
                    style={{ color: 'var(--text-tertiary)', width: 18, height: 18 }}
                  >
                    <Trash2 size={11} />
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      <div
        className="flex items-center"
        style={{
          padding: isExpanded ? '8px 12px' : '8px 4px',
          borderTop: '1px solid var(--border-primary)',
          justifyContent: isExpanded ? 'space-between' : 'center',
          gap: 4,
        }}
      >
        <button
          type="button"
          onClick={() => {
            const name = window.prompt('新建案件名称', '新案件')?.trim();
            if (name) void createCase(name);
          }}
          style={{
            height: 28,
            padding: isExpanded ? '0 10px' : '0 8px',
            borderRadius: 6,
            backgroundColor: 'var(--accent-primary)',
            color: 'var(--text-inverse)',
            fontSize: 11,
            display: 'flex',
            alignItems: 'center',
            gap: 4,
          }}
        >
          <Plus size={14} />
          {isExpanded ? '新建案件' : null}
        </button>
        <div className="flex" style={{ gap: 2 }}>
          <button
            type="button"
            onClick={() => void createSession()}
            title="新建会话"
            style={{ width: 28, height: 28, color: 'var(--text-tertiary)' }}
          >
            <MessageSquare size={16} />
          </button>
          <button
            type="button"
            onClick={() => navigate('/settings')}
            title="设置"
            style={{ width: 28, height: 28, color: 'var(--text-tertiary)' }}
          >
            <Settings size={16} />
          </button>
          <button type="button" title="帮助" style={{ width: 28, height: 28, color: 'var(--text-tertiary)' }}>
            <HelpCircle size={16} />
          </button>
        </div>
      </div>
    </div>
  );
};

export default ExplorerSidebar;
