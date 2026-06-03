import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { api, isAgentBackend, isHttpServerRuntime, isTauriRuntime } from '@/api';
import type {
  PatentCase,
  SessionMeta,
  StreamEvent,
  UsageSummary,
  WorkspaceProjectEntry,
  YunxiSettings,
} from '@/api';
import type { MaterialFileEntry } from '@/api/types';
import type { ImportMaterialsPreview } from '@/components/workbench/ImportMaterialsDialog';
import {
  chatConversation,
  patentCases as mockCases,
  sampleClaims,
  sessions as mockSessions,
} from '@/data/mockData';
import type {
  ChatMessage,
  PatentCase as MockPatentCase,
  PendingPermission,
  ToolCallBlock,
  ViewType,
} from '@/data/mockData';
import {
  parseSessionToMessages,
  sessionTitleFromJson,
  formatSessionTime,
} from '@/utils/sessionParse';
import { runSlashCommand, type SlashHandleResult } from '@/utils/slashCommandRunner';
import type {
  BottomPanelTab,
  DocxMode,
  DocxModeMap,
  EditorTab,
  PanelLogLevel,
  PanelLogLine,
  PanelProblem,
  SidebarActivity,
  WorkspaceFolder,
} from '@/types/workspace';
import {
  documentTabId,
  toolTabId,
  viewFromDocType,
} from '@/types/workspace';
import {
  folderLabelFromPath,
  loadPanelHeight,
  loadArchivedPaths,
  loadWorkspaceFolders,
  loadWorkspaceScanDepth,
  loadWorkspaceWatchEnabled,
  newFolderId,
  saveArchivedPaths,
  saveWorkspaceFolders,
  saveWorkspaceScanDepth,
  saveWorkspaceWatchEnabled,
} from '@/utils/workspaceStorage';
import { viewLabels } from '@/data/mockData';
import {
  defaultYunxiSettings,
  getDesktop,
  readBudgetUsd,
  withDesktopSection,
  type DesktopSettingsBlob,
} from '@/utils/desktopSettings';

export interface SessionListItem {
  id: string;
  title: string;
  timestamp: string;
  messageCount: number;
}

interface AppContextValue {
  ready: boolean;
  initError: string | null;
  isTauri: boolean;

  cases: MockPatentCase[];
  casesLoading: boolean;
  activeCaseId: string | null;
  activeDocId: string | null;
  activeDocContent: string;
  selectCase: (caseId: string) => void;
  /** @deprecated 请用 openDocument */
  selectDocument: (caseId: string, docId: string, docType?: string) => void;
  openDocument: (caseId: string, docId: string, docType: string, title: string) => void;
  openToolView: (view: ViewType) => void;
  editorTabs: EditorTab[];
  activeTabId: string | null;
  setActiveTab: (tabId: string) => void;
  closeEditorTab: (tabId: string) => void;
  sidebarActivity: SidebarActivity;
  setSidebarActivity: (activity: SidebarActivity) => void;
  createCase: (name: string) => Promise<void>;
  deleteCase: (caseId: string) => Promise<void>;
  archiveCase: (caseId: string) => Promise<void>;
  restoreCase: (caseId: string) => Promise<void>;
  archivedCases: MockPatentCase[];
  refreshCases: () => Promise<void>;

  sessions: SessionListItem[];
  activeSessionId: string | null;
  selectSession: (sessionId: string) => Promise<void>;
  createSession: (title?: string) => Promise<void>;
  deleteSession: (sessionId: string) => Promise<void>;

  messages: ChatMessage[];
  send: (
    content: string,
    caseId?: string,
    opts?: { skipSlash?: boolean; skipUserMessage?: boolean },
  ) => Promise<void>;
  cancel: () => void;
  isStreaming: boolean;
  chatError: string | null;

  usage: UsageSummary | null;
  model: string;
  budgetTotal: number;
  refreshUsage: () => Promise<void>;
  saveModel: (model: string) => Promise<void>;
  yunxiSettings: YunxiSettings | null;
  settingsReady: boolean;
  reloadYunxiSettings: () => Promise<void>;
  persistYunxiSettings: (next: YunxiSettings) => Promise<void>;
  updateDesktopSection: <K extends keyof DesktopSettingsBlob>(
    section: K,
    patch: Partial<NonNullable<DesktopSettingsBlob[K]>>,
  ) => Promise<void>;

  activeView: ViewType;
  setActiveView: (view: ViewType) => void;

  pendingPermission: PendingPermission | null;
  respondPermission: (outcome: 'allow' | 'deny' | 'always') => Promise<void>;
  toggleMessageReasoning: (messageId: string) => void;

  activeCase: PatentCase | undefined;
  getDocumentByType: (docType: string) => PatentCase['documents'][number] | undefined;
  updateCaseDocument: (docId: string, contentMd: string) => Promise<void>;
  docxMode: DocxMode;
  setDocxMode: (mode: DocxMode) => void;
  getDocxMode: (docId: string) => DocxMode;

  bottomPanelVisible: boolean;
  bottomPanelHeight: number;
  bottomPanelTab: BottomPanelTab;
  setBottomPanelVisible: (visible: boolean) => void;
  setBottomPanelHeight: (height: number) => void;
  setBottomPanelTab: (tab: BottomPanelTab) => void;
  toggleBottomPanel: (tab?: BottomPanelTab) => void;
  commandPaletteOpen: boolean;
  setCommandPaletteOpen: (open: boolean) => void;
  toggleCommandPalette: () => void;
  panelLogs: PanelLogLine[];
  panelProblems: PanelProblem[];
  problemCount: number;
  appendPanelLog: (message: string, level?: PanelLogLevel, source?: string) => void;
  terminalLines: string[];
  appendTerminalLine: (line: string) => void;
  appendTerminalChunk: (chunk: string) => void;

  workspaceFolders: WorkspaceFolder[];
  workspaceScanMaxDepth: number;
  setWorkspaceScanMaxDepth: (depth: number) => void;
  activeWorkspaceFolderId: string | null;
  activeWorkspaceFolder: WorkspaceFolder | undefined;
  addWorkspaceFolder: (path: string, label?: string) => void;
  removeWorkspaceFolder: (id: string) => void;
  archiveWorkspacePath: (path: string) => void;
  restoreArchivedPath: (path: string) => void;
  archivedPaths: string[];
  visibleWorkspaceFolders: WorkspaceFolder[];
  visibleWorkspaceProjects: WorkspaceProjectEntry[];
  setActiveWorkspaceFolder: (id: string) => void;
  pickWorkspaceFolderDialog: () => Promise<void>;
  workspaceProjects: WorkspaceProjectEntry[];
  workspaceWatchEnabled: boolean;
  setWorkspaceWatchEnabled: (enabled: boolean) => void;
  workspaceScanning: boolean;
  refreshWorkspaceScan: (options?: {
    silent?: boolean;
    folders?: WorkspaceFolder[];
  }) => Promise<void>;
  openWorkspaceProject: (project: WorkspaceProjectEntry) => void;
  /** 扫描项目材料并弹出确认对话框 */
  startImportProjectMaterials: (caseId: string, projectFolder: string) => Promise<void>;
  importMaterialsPreview: ImportMaterialsPreview | null;
  importMaterialsLoading: boolean;
  dismissImportMaterialsPreview: () => void;
  confirmImportMaterialsPreview: () => Promise<void>;
  /** 执行 slash 命令；默认同时写入输出面板与 AI 对话 */
  executeSlashCommand: (
    text: string,
    options?: { toChat?: boolean; toOutput?: boolean },
  ) => Promise<SlashHandleResult>;

  reorderEditorTabs: (fromIndex: number, toIndex: number) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

function nowLabel(): string {
  return new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
}

function nextLogId(): string {
  return `log-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
}

function projectsSnapshot(projects: WorkspaceProjectEntry[]): string {
  return JSON.stringify(
    projects.map((p) => ({
      folderPath: p.folderPath,
      caseId: p.caseId,
      isPatentProject: p.isPatentProject,
    })),
  );
}

function applyStreamEvent(message: ChatMessage, event: StreamEvent): ChatMessage {
  if (event.type === 'text_delta') {
    return { ...message, content: message.content + event.content };
  }
  if (event.type === 'reasoning_delta') {
    return {
      ...message,
      reasoning: (message.reasoning ?? '') + event.content,
      reasoningExpanded: message.reasoningExpanded ?? false,
    };
  }
  if (event.type === 'tool_use') {
    const tools: ToolCallBlock[] = [...(message.toolCalls ?? [])];
    const existing = tools.findIndex((t) => t.id === event.id);
    const block: ToolCallBlock = {
      id: event.id,
      name: event.name,
      input: event.input,
      status: 'running',
    };
    if (existing >= 0) tools[existing] = { ...tools[existing], ...block };
    else tools.push(block);
    return { ...message, toolCalls: tools };
  }
  if (event.type === 'tool_result') {
    const tools: ToolCallBlock[] = [...(message.toolCalls ?? [])];
    const idx = tools.findIndex((t) => t.id === event.id);
    const patch: ToolCallBlock = {
      id: event.id,
      name: idx >= 0 ? tools[idx].name : 'tool',
      output: event.output,
      isError: event.is_error,
      status: event.is_error ? 'error' : 'done',
    };
    if (idx >= 0) tools[idx] = { ...tools[idx], ...patch };
    else tools.push(patch);
    return { ...message, toolCalls: tools };
  }
  return message;
}

function mapCaseToUi(c: PatentCase): MockPatentCase {
  return {
    id: c.id,
    name: c.name,
    number: c.applicationNumber || '—',
    status: (c.status as MockPatentCase['status']) || 'draft',
    children: c.documents.map((d: PatentCase['documents'][number]) => ({
      id: d.id,
      name: d.title,
      type: d.type as MockPatentCase['children'][0]['type'],
    })),
  };
}

function docContentFromCase(c: PatentCase | undefined, docId: string | null): string {
  if (!c || !docId) return sampleClaims;
  const doc = c.documents.find((d: PatentCase['documents'][number]) => d.id === docId);
  return doc?.contentMd || sampleClaims;
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [ready, setReady] = useState(!isAgentBackend());
  const [initError, setInitError] = useState<string | null>(null);

  const [cases, setCases] = useState<MockPatentCase[]>(isAgentBackend() ? [] : mockCases);
  const [casesRaw, setCasesRaw] = useState<PatentCase[]>([]);
  const [casesLoading, setCasesLoading] = useState(isTauriRuntime());
  const [activeCaseId, setActiveCaseId] = useState<string | null>(
    isAgentBackend() ? null : 'case-1',
  );
  const [activeDocId, setActiveDocId] = useState<string | null>(
    isAgentBackend() ? null : 'c1-1',
  );

  const [sessions, setSessions] = useState<SessionListItem[]>(
    isAgentBackend()
      ? []
      : mockSessions.map((s) => ({ ...s, messageCount: 0 })),
  );
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>(
    isAgentBackend() ? [] : chatConversation,
  );
  const [isStreaming, setIsStreaming] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);

  const [usage, setUsage] = useState<UsageSummary | null>(null);
  const [model, setModel] = useState('deepseek-v4-pro');
  const [yunxiSettings, setYunxiSettings] = useState<YunxiSettings | null>(null);
  const [settingsReady, setSettingsReady] = useState(false);
  const [budgetTotal, setBudgetTotal] = useState(50);
  const [activeView, setActiveView] = useState<ViewType>('claims');
  const [editorTabs, setEditorTabs] = useState<EditorTab[]>(() =>
    isAgentBackend()
      ? []
      : [
          {
            id: documentTabId('case-1', 'c1-1'),
            title: '权利要求书',
            view: 'claims',
            kind: 'document',
            caseId: 'case-1',
            docId: 'c1-1',
          },
        ],
  );
  const [activeTabId, setActiveTabId] = useState<string | null>(() =>
    isAgentBackend() ? null : documentTabId('case-1', 'c1-1'),
  );
  const [sidebarActivity, setSidebarActivity] = useState<SidebarActivity>('explorer');
  const [bottomPanelVisible, setBottomPanelVisible] = useState(true);
  const [bottomPanelHeight, setBottomPanelHeight] = useState(() => loadPanelHeight(180));
  const [bottomPanelTab, setBottomPanelTab] = useState<BottomPanelTab>('output');
  const [panelLogs, setPanelLogs] = useState<PanelLogLine[]>([]);
  const [panelProblems] = useState<PanelProblem[]>([]);
  const [terminalLines, setTerminalLines] = useState<string[]>([]);
  const [workspaceFolders, setWorkspaceFolders] = useState<WorkspaceFolder[]>([]);
  const [activeWorkspaceFolderId, setActiveWorkspaceFolderId] = useState<string | null>(
    null,
  );
  const [workspaceProjects, setWorkspaceProjects] = useState<WorkspaceProjectEntry[]>([]);
  const [archivedPaths, setArchivedPaths] = useState<string[]>(() => loadArchivedPaths());
  const [workspaceWatchEnabled, setWorkspaceWatchEnabledState] = useState(() =>
    loadWorkspaceWatchEnabled(false),
  );
  const [workspaceScanning, setWorkspaceScanning] = useState(false);
  const [workspaceScanMaxDepth, setWorkspaceScanMaxDepthState] = useState(() =>
    loadWorkspaceScanDepth(2),
  );
  const [pendingPermission, setPendingPermission] = useState<PendingPermission | null>(
    null,
  );
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [importMaterialsPreview, setImportMaterialsPreview] =
    useState<ImportMaterialsPreview | null>(null);
  const [importMaterialsLoading, setImportMaterialsLoading] = useState(false);

  const [docxModes, setDocxModes] = useState<DocxModeMap>(() => {
    try {
      const raw = localStorage.getItem('yunxi-docx-modes')
      return raw ? JSON.parse(raw) : {}
    } catch {
      return {}
    }
  })

  const assistantIdRef = useRef<string | null>(null);
  const sendRef = useRef<
    (
      content: string,
      caseId?: string,
      opts?: { skipSlash?: boolean; skipUserMessage?: boolean },
    ) => Promise<void>
  >(() => Promise.resolve());
  const unlistenRef = useRef<(() => void) | null>(null);
  const bootstrappedRef = useRef(false);
  const workspaceWatchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const activeCase = useMemo(
    () => casesRaw.find((c) => c.id === activeCaseId),
    [casesRaw, activeCaseId],
  );

  const archivedSet = useMemo(() => new Set(archivedPaths), [archivedPaths]);

  const visibleCases = useMemo(
    () => cases.filter((c) => c.status !== 'archived'),
    [cases],
  );

  const archivedCases = useMemo(
    () => cases.filter((c) => c.status === 'archived'),
    [cases],
  );

  const visibleWorkspaceFolders = useMemo(
    () => workspaceFolders.filter((f) => !archivedSet.has(f.path)),
    [workspaceFolders, archivedSet],
  );

  const visibleWorkspaceProjects = useMemo(
    () => workspaceProjects.filter((p) => !archivedSet.has(p.folderPath)),
    [workspaceProjects, archivedSet],
  );

  const activeWorkspaceFolder = useMemo(
    () => workspaceFolders.find((f) => f.id === activeWorkspaceFolderId),
    [workspaceFolders, activeWorkspaceFolderId],
  );

  const setWorkspaceWatchEnabled = useCallback((enabled: boolean) => {
    setWorkspaceWatchEnabledState(enabled);
    saveWorkspaceWatchEnabled(enabled);
  }, []);

  const problemCount = useMemo(() => {
    let n = panelProblems.length;
    if (initError) n += 1;
    if (chatError) n += 1;
    return n;
  }, [panelProblems, initError, chatError]);

  const appendPanelLog = useCallback(
    (message: string, level: PanelLogLevel = 'info', source?: string) => {
      setPanelLogs((prev) => [
        ...prev.slice(-199),
        { id: nextLogId(), time: nowLabel(), level, message, source },
      ]);
    },
    [],
  );

  const appendTerminalLine = useCallback((line: string) => {
    if (line === '__CLEAR__') {
      setTerminalLines([]);
      return;
    }
    setTerminalLines((prev) => [...prev.slice(-300), line]);
  }, []);

  const appendTerminalChunk = useCallback((chunk: string) => {
    if (!chunk) return;
    setTerminalLines((prev) => {
      if (prev.length === 0) return [chunk];
      const last = prev[prev.length - 1];
      return [...prev.slice(0, -1), last + chunk];
    });
  }, []);

  const setWorkspaceScanMaxDepth = useCallback((depth: number) => {
    const clamped = Math.min(5, Math.max(1, Math.round(depth)));
    setWorkspaceScanMaxDepthState(clamped);
    saveWorkspaceScanDepth(clamped);
  }, []);

  const toggleBottomPanel = useCallback((tab?: BottomPanelTab) => {
    if (tab) {
      setBottomPanelTab(tab);
      setBottomPanelVisible(true);
      return;
    }
    setBottomPanelVisible((v) => !v);
  }, []);

  const toggleCommandPalette = useCallback(() => {
    setCommandPaletteOpen((v) => !v);
  }, []);

  const persistWorkspaceFolders = useCallback((folders: WorkspaceFolder[]) => {
    setWorkspaceFolders(folders);
    saveWorkspaceFolders(folders);
  }, []);

  const addWorkspaceFolder = useCallback(
    (path: string, label?: string) => {
      const trimmed = path.trim();
      if (!trimmed) return;
      if (workspaceFolders.some((f) => f.path === trimmed)) {
        appendPanelLog('该文件夹已在工作区中', 'warn', '工作区');
        return;
      }
      const folder: WorkspaceFolder = {
        id: newFolderId(),
        path: trimmed,
        label: label?.trim() || folderLabelFromPath(trimmed),
      };
      const next = [...workspaceFolders, folder];
      persistWorkspaceFolders(next);
      setActiveWorkspaceFolderId(folder.id);
      appendPanelLog(`已添加工作区文件夹: ${folder.label}`, 'info', '工作区');
    },
    [workspaceFolders, persistWorkspaceFolders, appendPanelLog],
  );

  const removeWorkspaceFolder = useCallback(
    (id: string) => {
      const target = workspaceFolders.find((f) => f.id === id);
      if (target?.isPrimary) {
        appendPanelLog('无法移除主工作区', 'warn', '工作区');
        return;
      }
      const next = workspaceFolders.filter((f) => f.id !== id);
      persistWorkspaceFolders(next);
      if (target) {
        setArchivedPaths((prev) => {
          const filtered = prev.filter((p) => p !== target.path);
          if (filtered.length !== prev.length) {
            saveArchivedPaths(filtered);
          }
          return filtered;
        });
      }
      if (activeWorkspaceFolderId === id) {
        setActiveWorkspaceFolderId(next[0]?.id ?? null);
      }
      if (target) {
        appendPanelLog(`已从工作区移除: ${target.label}`, 'info', '工作区');
      }
    },
    [workspaceFolders, activeWorkspaceFolderId, persistWorkspaceFolders, appendPanelLog],
  );

  const archiveWorkspacePath = useCallback(
    (path: string) => {
      const trimmed = path.trim();
      if (!trimmed || archivedSet.has(trimmed)) return;
      setArchivedPaths((prev) => {
        const next = [...prev, trimmed];
        saveArchivedPaths(next);
        return next;
      });
      appendPanelLog(`已归档: ${folderLabelFromPath(trimmed)}`, 'info', '工作区');
    },
    [archivedSet, appendPanelLog],
  );

  const restoreArchivedPath = useCallback(
    (path: string) => {
      setArchivedPaths((prev) => {
        const next = prev.filter((p) => p !== path);
        saveArchivedPaths(next);
        return next;
      });
      appendPanelLog(`已恢复: ${folderLabelFromPath(path)}`, 'info', '工作区');
    },
    [appendPanelLog],
  );

  const setActiveWorkspaceFolder = useCallback((id: string) => {
    setActiveWorkspaceFolderId(id);
  }, []);

  const refreshWorkspaceScan = useCallback(
    async (options?: { silent?: boolean; folders?: WorkspaceFolder[] }) => {
      const silent = options?.silent ?? false;
      const folders = options?.folders ?? workspaceFolders;
      if (folders.length === 0) {
        setWorkspaceProjects([]);
        return;
      }
      if (!silent) {
        setWorkspaceScanning(true);
      }
      try {
        const result = await api.scanWorkspaceRoots(
          folders.map((f) => f.path),
          workspaceScanMaxDepth,
        );
        let changed = false;
        setWorkspaceProjects((prev) => {
          const next = result.projects;
          if (projectsSnapshot(prev) === projectsSnapshot(next)) {
            return prev;
          }
          changed = true;
          return next;
        });
        if (!silent && changed) {
          appendPanelLog(
            `工作区扫描（深度 ${workspaceScanMaxDepth}）：发现 ${result.projects.length} 个项目`,
            'info',
            '工作区',
          );
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        if (!silent) {
          appendPanelLog(msg, 'error', '工作区');
        }
      } finally {
        if (!silent) {
          setWorkspaceScanning(false);
        }
      }
    },
    [workspaceFolders, workspaceScanMaxDepth, appendPanelLog],
  );

  const pickWorkspaceFolderDialog = useCallback(async () => {
    let path: string | null = null;
    if (isTauriRuntime()) {
      try {
        path = await api.pickWorkspaceFolder();
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendPanelLog(msg, 'error', '工作区');
      }
    }
    if (!path) {
      const manual = window.prompt('输入要添加的文件夹绝对路径：');
      path = manual?.trim() || null;
    }
    if (path) {
      addWorkspaceFolder(path);
      await refreshWorkspaceScan();
    }
  }, [addWorkspaceFolder, refreshWorkspaceScan, appendPanelLog]);

  const reorderEditorTabs = useCallback((fromIndex: number, toIndex: number) => {
    setEditorTabs((prev) => {
      if (
        fromIndex < 0 ||
        toIndex < 0 ||
        fromIndex >= prev.length ||
        toIndex >= prev.length ||
        fromIndex === toIndex
      ) {
        return prev;
      }
      const next = [...prev];
      const [moved] = next.splice(fromIndex, 1);
      next.splice(toIndex, 0, moved);
      return next;
    });
  }, []);

  const activeDocContent = useMemo(() => {
    return docContentFromCase(activeCase, activeDocId);
  }, [activeCase, activeDocId]);

  const getDocumentByType = useCallback(
    (docType: string) => activeCase?.documents.find((d) => d.type === docType),
    [activeCase],
  );

  const updateCaseDocument = useCallback(
    async (docId: string, contentMd: string) => {
      if (!isAgentBackend() || !activeCase) return;
      const now = String(Math.floor(Date.now() / 1000));
      const next: PatentCase = {
        ...activeCase,
        updatedAt: now,
        documents: activeCase.documents.map((d) =>
          d.id === docId ? { ...d, contentMd, updatedAt: now } : d,
        ),
      };
      const saved = await api.caseSave(next);
      setCasesRaw((prev) => prev.map((c) => (c.id === saved.id ? saved : c)));
      setCases((prev) => prev.map((c) => (c.id === saved.id ? mapCaseToUi(saved) : c)));
    },
    [activeCase],
  );

  const docxMode = useMemo<DocxMode>(() => {
    if (!activeDocId) return 'markdown'
    return docxModes[activeDocId] || 'markdown'
  }, [docxModes, activeDocId])

  const setDocxMode = useCallback(
    (mode: DocxMode) => {
      if (!activeDocId) return
      setDocxModes((prev) => {
        const next = { ...prev, [activeDocId]: mode }
        localStorage.setItem('yunxi-docx-modes', JSON.stringify(next))
        return next
      })
    },
    [activeDocId],
  )

  const getDocxMode = useCallback(
    (docId: string) => docxModes[docId] || 'markdown',
    [docxModes],
  )

  const refreshUsage = useCallback(async () => {
    try {
      const u = await api.getUsage();
      setUsage(u);
    } catch {
      // ignore
    }
  }, []);

  const applySettingsState = useCallback((settings: YunxiSettings) => {
    setYunxiSettings(settings);
    setModel(settings.model);
    setBudgetTotal(readBudgetUsd(settings, 50));
  }, []);

  const reloadYunxiSettings = useCallback(async () => {
    try {
      const settings = isAgentBackend()
        ? await api.getSettings()
        : defaultYunxiSettings(model);
      applySettingsState(settings);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      appendPanelLog(msg, 'warn', '设置');
    } finally {
      setSettingsReady(true);
    }
  }, [applySettingsState, model, appendPanelLog]);

  const persistYunxiSettings = useCallback(
    async (next: YunxiSettings) => {
      applySettingsState(next);
      if (!isAgentBackend()) return;
      try {
        await api.saveSettings(next);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendPanelLog(msg, 'error', '设置');
        throw e;
      }
    },
    [applySettingsState, appendPanelLog],
  );

  const updateDesktopSection = useCallback(
    async <K extends keyof DesktopSettingsBlob>(
      section: K,
      patch: Partial<NonNullable<DesktopSettingsBlob[K]>>,
    ) => {
      const base = yunxiSettings ?? defaultYunxiSettings(model);
      const next = withDesktopSection(base, section, patch);
      await persistYunxiSettings(next);
    },
    [yunxiSettings, model, persistYunxiSettings],
  );

  const saveModel = useCallback(
    async (nextModel: string) => {
      const base = yunxiSettings ?? defaultYunxiSettings(nextModel);
      await persistYunxiSettings({ ...base, model: nextModel });
    },
    [yunxiSettings, persistYunxiSettings],
  );

  const openDocument = useCallback(
    (caseId: string, docId: string, docType: string, title: string) => {
      const view = viewFromDocType(docType);
      const id = documentTabId(caseId, docId);
      setActiveCaseId(caseId);
      setActiveDocId(docId);
      setActiveView(view);
      setActiveTabId(id);
      setEditorTabs((prev) => {
        if (prev.some((t) => t.id === id)) return prev;
        return [
          ...prev,
          { id, title, view, kind: 'document', caseId, docId },
        ];
      });
    },
    [],
  );

  const openToolView = useCallback((view: ViewType) => {
    const id = toolTabId(view);
    setActiveView(view);
    setActiveTabId(id);
    setEditorTabs((prev) => {
      if (prev.some((t) => t.id === id)) return prev;
      return [...prev, { id, title: viewLabels[view], view, kind: 'tool' }];
    });
  }, []);

  const setActiveTab = useCallback((tabId: string) => {
    setActiveTabId(tabId);
    setEditorTabs((prev) => {
      const tab = prev.find((t) => t.id === tabId);
      if (tab) {
        setActiveView(tab.view);
        if (tab.caseId) setActiveCaseId(tab.caseId);
        if (tab.docId) setActiveDocId(tab.docId);
      }
      return prev;
    });
  }, []);

  const closeEditorTab = useCallback(
    (tabId: string) => {
      setEditorTabs((prev) => {
        const next = prev.filter((t) => t.id !== tabId);
        if (activeTabId === tabId) {
          const fallback = next[next.length - 1];
          if (fallback) {
            setActiveTabId(fallback.id);
            setActiveView(fallback.view);
            if (fallback.caseId) setActiveCaseId(fallback.caseId);
            if (fallback.docId) setActiveDocId(fallback.docId);
          } else {
            setActiveTabId(null);
          }
        }
        return next;
      });
    },
    [activeTabId],
  );

  const closeCaseTabs = useCallback(
    (caseId: string) => {
      setEditorTabs((prev) => {
        const next = prev.filter((t) => t.caseId !== caseId);
        if (activeTabId && prev.some((t) => t.id === activeTabId && t.caseId === caseId)) {
          const fallback = next[next.length - 1];
          if (fallback) {
            setActiveTabId(fallback.id);
            setActiveView(fallback.view);
            if (fallback.caseId) setActiveCaseId(fallback.caseId);
            if (fallback.docId) setActiveDocId(fallback.docId);
          } else {
            setActiveTabId(null);
            setActiveCaseId(null);
            setActiveDocId(null);
          }
        }
        return next;
      });
    },
    [activeTabId],
  );

  const refreshCases = useCallback(async () => {
    if (!isAgentBackend()) return;
    const showLoader = casesRaw.length === 0;
    if (showLoader) {
      setCasesLoading(true);
    }
    try {
      const list = await api.caseList();
      setCasesRaw(list);
      setCases(list.map(mapCaseToUi));
      const activeList = list.filter((c) => c.status !== 'archived');
      setActiveCaseId((prev) => {
        if (prev && activeList.some((c) => c.id === prev)) return prev;
        return activeList[0]?.id ?? null;
      });
      setEditorTabs((tabs) => {
        const filtered = tabs.filter(
          (t) => !t.caseId || activeList.some((c) => c.id === t.caseId),
        );
        if (filtered.length > 0) return filtered;
        const first = activeList[0];
        if (!first) return [];
        const doc =
          first.documents.find((d: PatentCase['documents'][number]) => d.type === 'claims') ??
          first.documents[0];
        if (doc) {
          const view = viewFromDocType(doc.type);
          const id = documentTabId(first.id, doc.id);
          setActiveDocId(doc.id);
          setActiveView(view);
          setActiveTabId(id);
          return [
            {
              id,
              title: doc.title,
              view,
              kind: 'document',
              caseId: first.id,
              docId: doc.id,
            },
          ];
        }
        return [];
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setInitError(msg);
      appendPanelLog(msg, 'error', '案件');
    } finally {
      if (showLoader) {
        setCasesLoading(false);
      }
    }
  }, [appendPanelLog, casesRaw.length]);

  const loadSessions = useCallback(async () => {
    if (!isAgentBackend()) return [];
    const metas: SessionMeta[] = await api.sessionList();
    const items: SessionListItem[] = [];
    for (const meta of metas) {
      let title = meta.id;
      try {
        const json = await api.sessionLoad(meta.id);
        title = sessionTitleFromJson(json, meta.id);
      } catch {
        // keep id
      }
      items.push({
        id: meta.id,
        title,
        timestamp: formatSessionTime(meta.modified_at),
        messageCount: meta.message_count,
      });
    }
    setSessions(items);
    return items;
  }, []);

  const selectSession = useCallback(async (sessionId: string) => {
    setActiveSessionId(sessionId);
    setChatError(null);
    if (!isAgentBackend()) return;
    try {
      const json = await api.sessionLoad(sessionId);
      const meta = sessions.find((s) => s.id === sessionId);
      setMessages(parseSessionToMessages(json, meta ? undefined : undefined));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setChatError(msg);
      appendPanelLog(msg, 'error', '会话');
    }
  }, [sessions, appendPanelLog]);

  const createSession = useCallback(async (title = '新对话') => {
    if (!isAgentBackend()) {
      const id = `s-${Date.now()}`;
      setSessions((prev) => [
        { id, title, timestamp: nowLabel(), messageCount: 0 },
        ...prev,
      ]);
      setActiveSessionId(id);
      setMessages([]);
      return;
    }
    const created = await api.sessionCreate(title);
    setActiveSessionId(created.id);
    setMessages([]);
    await loadSessions();
  }, [loadSessions]);

  const deleteSession = useCallback(async (sessionId: string) => {
    if (!isAgentBackend()) {
      setSessions((prev) => prev.filter((s) => s.id !== sessionId));
      if (activeSessionId === sessionId) {
        const remaining = sessions.filter((s) => s.id !== sessionId);
        const fallback = remaining[0]?.id ?? null;
        setActiveSessionId(fallback);
        setMessages(fallback ? [] : []);
      }
      return;
    }
    await api.sessionDelete(sessionId);
    if (activeSessionId === sessionId) {
      setActiveSessionId(null);
      setMessages([]);
    }
    await loadSessions();
  }, [activeSessionId, sessions, loadSessions]);

  const selectCase = useCallback(
    (caseId: string) => {
      const raw = casesRaw.find((c) => c.id === caseId);
      const ui = cases.find((c) => c.id === caseId);
      const doc =
        raw?.documents.find((d: PatentCase['documents'][number]) => d.type === 'claims') ??
        raw?.documents[0];
      const child = ui?.children.find((c) => c.id === doc?.id);
      if (doc && child) {
        openDocument(caseId, doc.id, doc.type, child.name);
      } else {
        setActiveCaseId(caseId);
      }
    },
    [casesRaw, cases, openDocument],
  );

  const selectDocument = useCallback(
    (caseId: string, docId: string, docType?: string) => {
      const ui = cases.find((c) => c.id === caseId);
      const child = ui?.children.find((c) => c.id === docId);
      const type = docType ?? child?.type ?? 'claims';
      const title = child?.name ?? '文档';
      openDocument(caseId, docId, type, title);
    },
    [cases, openDocument],
  );

  const openWorkspaceProject = useCallback(
    (project: WorkspaceProjectEntry) => {
      const folder = workspaceFolders.find((f) => f.path === project.workspaceRoot);
      if (folder) setActiveWorkspaceFolder(folder.id);
      if (project.caseId) {
        selectCase(project.caseId);
        appendPanelLog(`打开专利项目：${project.label}`, 'info', '工作区');
      } else if (project.isPatentProject) {
        appendPanelLog(
          `${project.label} 含 patentCase 但未解析案件 ID，请检查 YUNXI.md`,
          'warn',
          '工作区',
        );
      } else {
        appendPanelLog(`打开项目目录：${project.label}`, 'info', '工作区');
      }
    },
    [workspaceFolders, selectCase, appendPanelLog, setActiveWorkspaceFolder],
  );

  const createCase = useCallback(async (name: string) => {
    if (!isTauriRuntime()) {
      const id = `case-${Date.now()}`;
      const next: MockPatentCase = {
        id,
        name,
        number: '—',
        status: 'draft',
        children: [],
      };
      setCases((prev) => [next, ...prev]);
      setActiveCaseId(id);
      appendPanelLog(`已创建案件：${name}`, 'info', '案件');
      return;
    }
    const created = await api.caseCreate(name);
    await refreshCases();
    setActiveCaseId(created.id);
    appendPanelLog(`已创建案件：${name}`, 'info', '案件');
  }, [refreshCases, appendPanelLog]);

  const deleteCase = useCallback(
    async (caseId: string) => {
      const label =
        cases.find((c) => c.id === caseId)?.name ??
        casesRaw.find((c) => c.id === caseId)?.name ??
        caseId;
      if (!isTauriRuntime()) {
        setCases((prev) => prev.filter((c) => c.id !== caseId));
        setCasesRaw((prev) => prev.filter((c) => c.id !== caseId));
        closeCaseTabs(caseId);
        if (activeCaseId === caseId) {
          setActiveCaseId(null);
          setActiveDocId(null);
        }
        appendPanelLog(`已删除案件：${label}`, 'info', '案件');
        return;
      }
      await api.caseDelete(caseId);
      closeCaseTabs(caseId);
      if (activeCaseId === caseId) {
        setActiveCaseId(null);
        setActiveDocId(null);
      }
      await refreshCases();
      appendPanelLog(`已删除案件：${label}`, 'info', '案件');
    },
    [cases, casesRaw, closeCaseTabs, activeCaseId, refreshCases, appendPanelLog],
  );

  const archiveCase = useCallback(
    async (caseId: string) => {
      const raw = casesRaw.find((c) => c.id === caseId);
      const label = raw?.name ?? cases.find((c) => c.id === caseId)?.name ?? caseId;
      if (!isTauriRuntime()) {
        setCases((prev) =>
          prev.map((c) => (c.id === caseId ? { ...c, status: 'archived' as const } : c)),
        );
        closeCaseTabs(caseId);
        if (activeCaseId === caseId) {
          setActiveCaseId(null);
          setActiveDocId(null);
        }
        appendPanelLog(`已归档案件：${label}`, 'info', '案件');
        return;
      }
      if (!raw) return;
      const now = String(Math.floor(Date.now() / 1000));
      await api.caseSave({ ...raw, status: 'archived', updatedAt: now });
      closeCaseTabs(caseId);
      if (activeCaseId === caseId) {
        setActiveCaseId(null);
        setActiveDocId(null);
      }
      await refreshCases();
      appendPanelLog(`已归档案件：${label}`, 'info', '案件');
    },
    [casesRaw, cases, closeCaseTabs, activeCaseId, refreshCases, appendPanelLog],
  );

  const restoreCase = useCallback(
    async (caseId: string) => {
      const raw = casesRaw.find((c) => c.id === caseId);
      const label = raw?.name ?? cases.find((c) => c.id === caseId)?.name ?? caseId;
      if (!isTauriRuntime()) {
        setCases((prev) =>
          prev.map((c) => (c.id === caseId ? { ...c, status: 'draft' as const } : c)),
        );
        appendPanelLog(`已恢复案件：${label}`, 'info', '案件');
        return;
      }
      if (!raw) return;
      const now = String(Math.floor(Date.now() / 1000));
      await api.caseSave({ ...raw, status: 'draft', updatedAt: now });
      await refreshCases();
      appendPanelLog(`已恢复案件：${label}`, 'info', '案件');
    },
    [casesRaw, cases, refreshCases, appendPanelLog],
  );

  const executeSlashCommand = useCallback(
    async (
      text: string,
      options?: { toChat?: boolean; toOutput?: boolean },
    ) => {
      const result = await runSlashCommand(
        text,
        activeSessionId,
        model,
        usage,
        activeWorkspaceFolder?.path,
      );
      if (result !== null) {
        if (result.kind === 'agent_turn') {
          const userMsg: ChatMessage = {
            id: `m-${Date.now()}`,
            role: 'user',
            content: text.trim(),
            timestamp: nowLabel(),
          };
          if (options?.toChat !== false) {
            setMessages((prev) => [...prev, userMsg]);
          }
          await sendRef.current(result.prompt, activeCase?.id ?? undefined, {
            skipSlash: true,
            skipUserMessage: true,
          });
          return result;
        }
        const reply =
          result.kind === 'message' || result.kind === 'session_updated'
            ? result.content
            : '';
        const toOutput = options?.toOutput !== false;
        const toChat = options?.toChat !== false;
        if (toOutput && reply) {
          const plain = reply.replace(/\*\*/g, '');
          appendPanelLog(
            plain.length > 500 ? `${plain.slice(0, 500)}…` : plain,
            'info',
            '命令',
          );
          setBottomPanelTab('output');
          setBottomPanelVisible(true);
        }
        if (toChat && reply) {
          const userMsg: ChatMessage = {
            id: `m-${Date.now()}`,
            role: 'user',
            content: text.trim(),
            timestamp: nowLabel(),
          };
          const aiMsg: ChatMessage = {
            id: `m-${Date.now()}-slash`,
            role: 'ai',
            content: reply,
            timestamp: nowLabel(),
          };
          setMessages((prev) => [...prev, userMsg, aiMsg]);
        }
        if (result.kind === 'session_updated') {
          try {
            setMessages(parseSessionToMessages(result.session_json));
          } catch {
            /* ignore */
          }
        }
      }
      return result;
    },
    [
      activeSessionId,
      activeWorkspaceFolder?.path,
      activeCase?.id,
      model,
      usage,
      appendPanelLog,
    ],
  );

  const importProjectMaterials = useCallback(
    async (caseId: string, projectFolder: string) => {
      if (!isTauriRuntime()) {
        appendPanelLog('导入材料需桌面运行时', 'warn', '工作区');
        return;
      }
      try {
        const depth =
          getDesktop(yunxiSettings).materials?.scanMaxDepth ?? workspaceScanMaxDepth;
        const result = await api.importProjectMaterials(
          caseId,
          projectFolder,
          undefined,
          depth,
        );
        setCasesRaw((prev) => prev.map((c) => (c.id === result.case.id ? result.case : c)));
        setCases((prev) =>
          prev.map((c) =>
            c.id === result.case.id ? mapCaseToUi(result.case) : c,
          ),
        );
        selectCase(caseId);
        const summary = [
          result.imported.length ? `导入 ${result.imported.length} 个` : null,
          result.skipped.length ? `跳过 ${result.skipped.length} 个` : null,
          result.errors.length ? `错误 ${result.errors.length} 个` : null,
        ]
          .filter(Boolean)
          .join('，');
        appendPanelLog(summary || '未导入任何材料', 'info', '工作区');
        if (result.errors.length) {
          appendPanelLog(result.errors.join('; '), 'warn', '工作区');
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendPanelLog(msg, 'error', '工作区');
      }
    },
    [appendPanelLog, selectCase, yunxiSettings, workspaceScanMaxDepth],
  );

  const startImportProjectMaterials = useCallback(
    async (caseId: string, projectFolder: string) => {
      if (!isTauriRuntime()) {
        appendPanelLog('导入材料需桌面运行时', 'warn', '工作区');
        return;
      }
      try {
        const depth =
          getDesktop(yunxiSettings).materials?.scanMaxDepth ?? workspaceScanMaxDepth;
        const files: MaterialFileEntry[] = await api.listProjectMaterials(
          projectFolder,
          depth,
        );
        if (files.length === 0) {
          appendPanelLog('未发现可导入的材料文件', 'warn', '工作区');
          return;
        }
        const caseName =
          casesRaw.find((c) => c.id === caseId)?.name ??
          cases.find((c) => c.id === caseId)?.name;
        setImportMaterialsPreview({
          caseId,
          caseName,
          projectFolder,
          files,
        });
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendPanelLog(msg, 'error', '工作区');
      }
    },
    [
      appendPanelLog,
      yunxiSettings,
      workspaceScanMaxDepth,
      casesRaw,
      cases,
    ],
  );

  const dismissImportMaterialsPreview = useCallback(() => {
    setImportMaterialsPreview(null);
  }, []);

  const confirmImportMaterialsPreview = useCallback(async () => {
    const preview = importMaterialsPreview;
    if (!preview || importMaterialsLoading) return;
    setImportMaterialsLoading(true);
    try {
      await importProjectMaterials(preview.caseId, preview.projectFolder);
      setImportMaterialsPreview(null);
    } finally {
      setImportMaterialsLoading(false);
    }
  }, [importMaterialsPreview, importMaterialsLoading, importProjectMaterials]);

  useEffect(() => {
    if (!workspaceWatchEnabled || !isTauriRuntime() || workspaceFolders.length === 0) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void (async () => {
      try {
        await api.workspaceWatchStart(workspaceFolders.map((f) => f.path));
        if (cancelled) return;
        unlisten = await api.onWorkspaceChanged(() => {
          if (workspaceWatchTimerRef.current) {
            clearTimeout(workspaceWatchTimerRef.current);
          }
          workspaceWatchTimerRef.current = setTimeout(() => {
            workspaceWatchTimerRef.current = null;
            void refreshWorkspaceScan({ silent: true });
          }, 1500);
        });
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendPanelLog(msg, 'warn', '工作区');
      }
    })();
    return () => {
      cancelled = true;
      if (workspaceWatchTimerRef.current) {
        clearTimeout(workspaceWatchTimerRef.current);
        workspaceWatchTimerRef.current = null;
      }
      unlisten?.();
      void api.workspaceWatchStop();
    };
  }, [workspaceWatchEnabled, workspaceFolders, refreshWorkspaceScan, appendPanelLog]);

  useEffect(() => {
    if (initError || chatError) {
      setBottomPanelTab('problems');
      setBottomPanelVisible(true);
    }
  }, [initError, chatError]);

  const initWorkspaceFolders = useCallback(async () => {
    const stored = loadWorkspaceFolders();
    let root = '.';
    if (isTauriRuntime()) {
      try {
        const info = await api.getWorkspaceInfo();
        root = info.workspaceRoot;
      } catch {
        // 使用默认路径
      }
    }
    const primary: WorkspaceFolder = {
      id: 'primary',
      path: root,
      label: folderLabelFromPath(root),
      isPrimary: true,
    };
    const merged = [
      primary,
      ...stored.filter((s) => s.path !== primary.path && s.id !== 'primary'),
    ];
    setWorkspaceFolders(merged);
    setActiveWorkspaceFolderId((prev) => prev ?? primary.id);
    return merged;
  }, []);

  useEffect(() => {
    if (bootstrappedRef.current) {
      return;
    }
    bootstrappedRef.current = true;

    if (!isAgentBackend()) {
      api.getUsage().then(setUsage).catch(() => {});
      api
        .getSettings()
        .then((s) => {
          applySettingsState(s);
          setSettingsReady(true);
        })
        .catch(() => {});
      if (!isHttpServerRuntime()) {
        void (async () => {
          const folders = await initWorkspaceFolders();
          await refreshWorkspaceScan({ folders });
        })();
        appendPanelLog('Web 预览模式已加载', 'info', '系统');
      }
      return;
    }

    let cancelled = false;
    (async () => {
      try {
        const [settings, usageSummary, sessionItems] = await Promise.all([
          api.getSettings(),
          api.getUsage(),
          loadSessions(),
        ]);
        if (cancelled) return;
        applySettingsState(settings);
        setSettingsReady(true);
        setUsage(usageSummary);
        if (isTauriRuntime()) {
          const folders = await initWorkspaceFolders();
          await refreshWorkspaceScan({ folders });
        }
        await refreshCases();

        let sid: string;
        if (sessionItems.length > 0) {
          sid = sessionItems[0].id;
        } else {
          const created = await api.sessionCreate('云熙对话');
          sid = created.id;
          await loadSessions();
        }
        if (cancelled) return;
        setActiveSessionId(sid);
        const json = await api.sessionLoad(sid);
        setMessages(parseSessionToMessages(json));
        setReady(true);
        appendPanelLog(
          isHttpServerRuntime() ? '云熙 HTTP Server 已就绪' : '云熙桌面已就绪',
          'info',
          '系统',
        );
      } catch (e) {
        if (!cancelled) {
          const msg = e instanceof Error ? e.message : String(e);
          setInitError(msg);
          appendPanelLog(msg, 'error', '初始化');
          setReady(true);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
    // 仅启动时引导一次；勿把 refreshWorkspaceScan 等放入 deps，否则会随 workspaceFolders 反复初始化
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const respondPermission = useCallback(async (outcome: 'allow' | 'deny' | 'always') => {
    const pending = pendingPermission;
    if (!pending) return;
    setPendingPermission(null);
    if (!isAgentBackend()) return;
    try {
      await api.permissionRespond(pending.requestId, outcome);
    } catch (e) {
      setChatError(e instanceof Error ? e.message : String(e));
    }
  }, [pendingPermission]);

  const toggleMessageReasoning = useCallback((messageId: string) => {
    setMessages((prev) =>
      prev.map((m) =>
        m.id === messageId
          ? { ...m, reasoningExpanded: !m.reasoningExpanded }
          : m,
      ),
    );
  }, []);

  const cancel = useCallback(() => {
    if (activeSessionId) void api.chatCancel(activeSessionId);
    unlistenRef.current?.();
    unlistenRef.current = null;
    setIsStreaming(false);
    const aid = assistantIdRef.current;
    if (aid) {
      setMessages((prev) => prev.map((m) => (m.id === aid ? { ...m, isStreaming: false } : m)));
    }
  }, [activeSessionId]);

  const send = useCallback(
    async (
      content: string,
      caseId?: string,
      opts?: { skipSlash?: boolean; skipUserMessage?: boolean },
    ) => {
      const text = content.trim();
      if (!text || isStreaming) return;

      setChatError(null);

      const wsRoot = activeWorkspaceFolder?.path;

      if (!opts?.skipSlash) {
        const slashResult = await runSlashCommand(
          text,
          activeSessionId,
          model,
          usage,
          wsRoot,
        );
        if (slashResult !== null) {
          if (slashResult.kind === 'agent_turn') {
            const userMsg: ChatMessage = {
              id: `m-${Date.now()}`,
              role: 'user',
              content: text,
              timestamp: nowLabel(),
            };
            setMessages((prev) => [...prev, userMsg]);
            await sendRef.current(slashResult.prompt, caseId, {
              skipSlash: true,
              skipUserMessage: true,
            });
            return;
          }

          if (!opts?.skipUserMessage) {
            const userMsg: ChatMessage = {
              id: `m-${Date.now()}`,
              role: 'user',
              content: text,
              timestamp: nowLabel(),
            };
            setMessages((prev) => [...prev, userMsg]);
          }

          if (slashResult.kind === 'message') {
            setMessages((prev) => [
              ...prev,
              {
                id: `m-${Date.now()}-slash`,
                role: 'ai',
                content: slashResult.content,
                timestamp: nowLabel(),
              },
            ]);
            return;
          }

          if (slashResult.kind === 'session_updated') {
            setMessages((prev) => [
              ...prev,
              {
                id: `m-${Date.now()}-slash`,
                role: 'ai',
                content: slashResult.content,
                timestamp: nowLabel(),
              },
            ]);
            try {
              const parsed = parseSessionToMessages(slashResult.session_json);
              setMessages(parsed);
            } catch {
              /* 保留文本回复 */
            }
            await loadSessions();
            return;
          }
        }
      }

      if (!isAgentBackend()) {
        const userMsg: ChatMessage = {
          id: `m-${Date.now()}`,
          role: 'user',
          content: text,
          timestamp: nowLabel(),
        };
        setMessages((prev) => [...prev, userMsg]);
        setIsStreaming(true);
        try {
          await api.chatSend('session-mock', text);
          setMessages((prev) => [
            ...prev,
            {
              id: `m-${Date.now()}-ai`,
              role: 'ai',
              content: '（Mock 模式）消息已收到。',
              timestamp: nowLabel(),
            },
          ]);
        } finally {
          setIsStreaming(false);
        }
        return;
      }

      if (!activeSessionId) {
        setChatError('会话尚未就绪');
        return;
      }

      const assistantId = `m-${Date.now()}-ai`;
      assistantIdRef.current = assistantId;
      const additions: ChatMessage[] = [];
      if (!opts?.skipUserMessage) {
        additions.push({
          id: `m-${Date.now()}`,
          role: 'user',
          content: text,
          timestamp: nowLabel(),
        });
      }
      additions.push({
        id: assistantId,
        role: 'ai',
        content: '',
        timestamp: nowLabel(),
        isStreaming: true,
      });
      setMessages((prev) => [...prev, ...additions]);

      setIsStreaming(true);

      const unlisten = await api.onStream(activeSessionId, (event) => {
        if (
          event.type === 'text_delta' ||
          event.type === 'reasoning_delta' ||
          event.type === 'tool_use' ||
          event.type === 'tool_result'
        ) {
          setMessages((prev) =>
            prev.map((m) =>
              m.id === assistantId ? applyStreamEvent(m, event) : m,
            ),
          );
        } else if (event.type === 'permission_request') {
          setPendingPermission({
            requestId: event.request_id,
            tool: event.tool,
            input: event.input,
          });
        } else if (event.type === 'usage') {
          void refreshUsage();
        } else if (event.type === 'error') {
          setChatError(event.message);
          appendPanelLog(event.message, 'error', '对话');
          setMessages((prev) =>
            prev.map((m) =>
              m.id === assistantId
                ? { ...m, content: m.content || `错误：${event.message}`, isStreaming: false }
                : m,
            ),
          );
          setIsStreaming(false);
        } else if (event.type === 'message_stop') {
          setMessages((prev) =>
            prev.map((m) => (m.id === assistantId ? { ...m, isStreaming: false } : m)),
          );
          setIsStreaming(false);
        }
      });
      unlistenRef.current = unlisten;

      try {
        const wsRoot = activeWorkspaceFolder?.path;
        await api.chatSend(activeSessionId, text, caseId ?? activeCaseId ?? undefined, wsRoot);
        await refreshUsage();
        await loadSessions();
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        setChatError(message);
        appendPanelLog(message, 'error', '对话');
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantId
              ? { ...m, content: m.content || `错误：${message}`, isStreaming: false }
              : m,
          ),
        );
      } finally {
        unlistenRef.current?.();
        unlistenRef.current = null;
        assistantIdRef.current = null;
        setIsStreaming(false);
        setMessages((prev) =>
          prev.map((m) => (m.id === assistantId ? { ...m, isStreaming: false } : m)),
        );
      }
    },
    [
      isStreaming,
      activeSessionId,
      activeCaseId,
      activeWorkspaceFolder,
      model,
      usage,
      refreshUsage,
      loadSessions,
      appendPanelLog,
    ],
  );

  sendRef.current = send;

  useEffect(
    () => () => {
      unlistenRef.current?.();
    },
    [],
  );

  const value = useMemo<AppContextValue>(
    () => ({
      ready,
      initError,
      isTauri: isTauriRuntime(),
      cases: visibleCases,
      casesLoading,
      activeCaseId,
      activeDocId,
      activeDocContent,
      selectCase,
      selectDocument,
      openDocument,
      openToolView,
      editorTabs,
      activeTabId,
      setActiveTab,
      closeEditorTab,
      sidebarActivity,
      setSidebarActivity,
      createCase,
      deleteCase,
      archiveCase,
      restoreCase,
      archivedCases,
      refreshCases,
      sessions,
      activeSessionId,
      selectSession,
      createSession,
      deleteSession,
      messages,
      send,
      cancel,
      isStreaming,
      chatError,
      usage,
      model,
      budgetTotal,
      refreshUsage,
      saveModel,
      yunxiSettings,
      settingsReady,
      reloadYunxiSettings,
      persistYunxiSettings,
      updateDesktopSection,
      activeView,
      setActiveView,
      pendingPermission,
      respondPermission,
      toggleMessageReasoning,
      activeCase,
      getDocumentByType,
      updateCaseDocument,
      bottomPanelVisible,
      bottomPanelHeight,
      bottomPanelTab,
      setBottomPanelVisible,
      setBottomPanelHeight,
      setBottomPanelTab,
      toggleBottomPanel,
      commandPaletteOpen,
      setCommandPaletteOpen,
      toggleCommandPalette,
      panelLogs,
      panelProblems,
      problemCount,
      appendPanelLog,
      terminalLines,
      appendTerminalLine,
      appendTerminalChunk,
      workspaceFolders,
      workspaceScanMaxDepth,
      setWorkspaceScanMaxDepth,
      activeWorkspaceFolderId,
      activeWorkspaceFolder,
      addWorkspaceFolder,
      removeWorkspaceFolder,
      archiveWorkspacePath,
      restoreArchivedPath,
      archivedPaths,
      visibleWorkspaceFolders,
      visibleWorkspaceProjects,
      setActiveWorkspaceFolder,
      pickWorkspaceFolderDialog,
      workspaceProjects,
      workspaceWatchEnabled,
      setWorkspaceWatchEnabled,
      workspaceScanning,
      refreshWorkspaceScan,
      openWorkspaceProject,
      startImportProjectMaterials,
      importMaterialsPreview,
      importMaterialsLoading,
      dismissImportMaterialsPreview,
      confirmImportMaterialsPreview,
      executeSlashCommand,
      reorderEditorTabs,
      docxMode,
      setDocxMode,
      getDocxMode,
    }),
    [
      ready,
      initError,
      visibleCases,
      archivedCases,
      casesLoading,
      activeCaseId,
      activeDocId,
      activeDocContent,
      selectCase,
      selectDocument,
      openDocument,
      openToolView,
      editorTabs,
      activeTabId,
      setActiveTab,
      closeEditorTab,
      sidebarActivity,
      createCase,
      deleteCase,
      archiveCase,
      restoreCase,
      refreshCases,
      sessions,
      activeSessionId,
      selectSession,
      createSession,
      deleteSession,
      messages,
      send,
      cancel,
      isStreaming,
      chatError,
      usage,
      model,
      budgetTotal,
      refreshUsage,
      saveModel,
      yunxiSettings,
      settingsReady,
      reloadYunxiSettings,
      persistYunxiSettings,
      updateDesktopSection,
      activeView,
      pendingPermission,
      respondPermission,
      toggleMessageReasoning,
      activeCase,
      getDocumentByType,
      updateCaseDocument,
      bottomPanelVisible,
      bottomPanelHeight,
      bottomPanelTab,
      setBottomPanelVisible,
      setBottomPanelHeight,
      setBottomPanelTab,
      toggleBottomPanel,
      commandPaletteOpen,
      setCommandPaletteOpen,
      toggleCommandPalette,
      panelLogs,
      panelProblems,
      problemCount,
      appendPanelLog,
      terminalLines,
      appendTerminalLine,
      appendTerminalChunk,
      toggleBottomPanel,
      commandPaletteOpen,
      setCommandPaletteOpen,
      toggleCommandPalette,
      workspaceFolders,
      workspaceScanMaxDepth,
      setWorkspaceScanMaxDepth,
      activeWorkspaceFolderId,
      activeWorkspaceFolder,
      addWorkspaceFolder,
      removeWorkspaceFolder,
      archiveWorkspacePath,
      restoreArchivedPath,
      archivedPaths,
      visibleWorkspaceFolders,
      visibleWorkspaceProjects,
      setActiveWorkspaceFolder,
      pickWorkspaceFolderDialog,
      workspaceProjects,
      workspaceWatchEnabled,
      setWorkspaceWatchEnabled,
      workspaceScanning,
      refreshWorkspaceScan,
      openWorkspaceProject,
      startImportProjectMaterials,
      importMaterialsPreview,
      importMaterialsLoading,
      dismissImportMaterialsPreview,
      confirmImportMaterialsPreview,
       executeSlashCommand,
      reorderEditorTabs,
      docxMode,
      setDocxMode,
      getDocxMode,
    ],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp 必须在 AppProvider 内使用');
  return ctx;
}
