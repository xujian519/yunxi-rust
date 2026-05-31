import type {
  ChatSendResult,
  PatentCase,
  SessionCreateResult,
  SessionMeta,
  SessionSaveResult,
  StreamEvent,
  UsageSummary,
  WorkspaceInfo,
  ScanWorkspaceResult,
  ShellExecResult,
  ShellEvent,
  MaterialFileEntry,
  ImportMaterialsResult,
  YunxiSettings,
} from './types';

type TauriGlobal = {
  core?: {
    invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
  };
  event?: {
    listen: <T>(
      event: string,
      handler: (event: { payload: T }) => void,
    ) => Promise<() => void>;
  };
  invoke?: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
};

declare global {
  interface Window {
    __TAURI__?: TauriGlobal;
  }
}

function tauri(): TauriGlobal | undefined {
  return window.__TAURI__;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const api = tauri();
  const invokeFn = api?.core?.invoke ?? api?.invoke;
  if (!invokeFn) {
    throw new Error('Tauri API 不可用');
  }
  try {
    return await invokeFn<T>(cmd, args);
  } catch (e) {
    throw new Error(formatInvokeError(e));
  }
}

function formatInvokeError(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error && e.message) return e.message;
  if (e && typeof e === 'object') {
    const obj = e as Record<string, unknown>;
    if (typeof obj.message === 'string') return obj.message;
    if (typeof obj.error === 'string') return obj.error;
  }
  return '未知错误';
}

function streamChannel(sessionId: string): string {
  return `yunxi://stream/${sessionId}`;
}

/** Tauri IPC API — 对接 desktop/commands */
export const tauriApi = {
  getVersion: () => invoke<string>('get_version'),
  getSettings: () => invoke<YunxiSettings>('get_settings'),
  saveSettings: (settings: YunxiSettings) => invoke<void>('save_settings', { settings }),
  getUsage: () => invoke<UsageSummary>('get_usage'),

  sessionList: () => invoke<SessionMeta[]>('session_list'),
  sessionLoad: (id: string) => invoke<string>('session_load', { id }),
  sessionSave: (sessionJson: string) =>
    invoke<SessionSaveResult>('session_save', { sessionJson }),
  sessionCreate: (title: string) =>
    invoke<SessionCreateResult>('session_create', { title }),

  chatSend: (sessionId: string, content: string, caseId?: string) =>
    invoke<ChatSendResult>('chat_send', { sessionId, content, caseId }),
  chatCancel: (sessionId: string) => invoke<void>('chat_cancel', { sessionId }),

  permissionRespond: (requestId: string, outcome: 'allow' | 'deny' | 'always') =>
    invoke<void>('permission_respond', { requestId, outcome }),

  getWorkspaceInfo: () => invoke<WorkspaceInfo>('get_workspace_info'),
  pickWorkspaceFolder: () => invoke<string | null>('pick_workspace_folder'),
  scanWorkspaceRoots: (paths: string[], maxDepth?: number) =>
    invoke<ScanWorkspaceResult>('scan_workspace_roots', { paths, maxDepth }),
  workspaceWatchStart: (paths: string[]) =>
    invoke<void>('workspace_watch_start', { paths }),
  workspaceWatchStop: () => invoke<void>('workspace_watch_stop'),
  shellExec: (workingDir: string, command: string) =>
    invoke<ShellExecResult>('shell_exec', { workingDir, command }),
  shellSessionStart: (workingDir: string) =>
    invoke<string>('shell_session_start', { workingDir }),
  shellSessionWrite: (sessionId: string, data: string) =>
    invoke<void>('shell_session_write', { sessionId, data }),
  shellSessionClose: (sessionId: string) =>
    invoke<void>('shell_session_close', { sessionId }),
  listProjectMaterials: (projectFolder: string, maxDepth?: number) =>
    invoke<MaterialFileEntry[]>('list_project_materials', { projectFolder, maxDepth }),
  llmAuthConfigured: (model?: string) =>
    invoke<boolean>('llm_auth_configured', { model }),
  saveLlmApiKey: (model: string, apiKey: string) =>
    invoke<YunxiSettings>('save_llm_api_key', { model, apiKey }),
  shellSessionResize: (sessionId: string, rows: number, cols: number) =>
    invoke<void>('shell_session_resize', { sessionId, rows, cols }),
  importProjectMaterials: (
    caseId: string,
    projectFolder: string,
    maxFiles?: number,
    maxDepth?: number,
  ) =>
    invoke<ImportMaterialsResult>('import_project_materials', {
      caseId,
      projectFolder,
      maxFiles,
      maxDepth,
    }),

  caseList: () => invoke<PatentCase[]>('case_list'),
  caseLoad: (id: string) => invoke<PatentCase>('case_load', { id }),
  caseSave: (caseData: PatentCase) => invoke<PatentCase>('case_save', { case: caseData }),
  caseCreate: (name: string, applicationNumber?: string) =>
    invoke<PatentCase>('case_create', { name, applicationNumber }),
  caseDelete: (id: string) => invoke<void>('case_delete', { id }),

  patentSearch: (query: string, limit?: number) =>
    invoke<string>('patent_search', { query, limit }),
  knowledgeSearch: (query: string) => invoke<string>('knowledge_search', { query }),

  /** 订阅流式事件，返回取消监听函数 */
  async onStream(
    sessionId: string,
    handler: (event: StreamEvent) => void,
  ): Promise<() => void> {
    const listen = tauri()?.event?.listen;
    if (!listen) {
      throw new Error('Tauri event API 不可用');
    }
    return listen<StreamEvent>(streamChannel(sessionId), (e) => handler(e.payload));
  },

  shellChannel(sessionId: string): string {
    return `yunxi://shell/${sessionId}`;
  },

  async onShell(sessionId: string, handler: (event: ShellEvent) => void): Promise<() => void> {
    const listen = tauri()?.event?.listen;
    if (!listen) {
      throw new Error('Tauri event API 不可用');
    }
    return listen<ShellEvent>(this.shellChannel(sessionId), (e) => handler(e.payload));
  },

  async onWorkspaceChanged(handler: () => void): Promise<() => void> {
    const listen = tauri()?.event?.listen;
    if (!listen) {
      throw new Error('Tauri event API 不可用');
    }
    return listen('yunxi://workspace/changed', () => handler());
  },
};

export type { StreamEvent, UsageSummary, YunxiSettings };
