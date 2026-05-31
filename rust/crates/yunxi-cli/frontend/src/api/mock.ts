import type { SessionMeta, StreamEvent, UsageSummary, YunxiSettings } from './types';

/** Mock API — 开发阶段使用 */
export const mockApi = {
  async getVersion(): Promise<string> {
    return '0.1.0-mock';
  },

  async getSettings(): Promise<YunxiSettings> {
    return { model: 'deepseek-v4-pro', permissions: { defaultMode: 'dontAsk' } };
  },

  async saveSettings(_settings: YunxiSettings): Promise<void> {
    /* mock no-op */
  },

  async getUsage(): Promise<UsageSummary> {
    return { input_tokens: 12000, output_tokens: 3400, estimated_cost: 2.35 };
  },

  async sessionList(): Promise<SessionMeta[]> {
    return [{ id: 'session-mock', message_count: 3, modified_at: Date.now() / 1000 }];
  },

  async sessionLoad(_id: string): Promise<string> {
    return JSON.stringify({ version: 1, messages: [] });
  },

  async sessionSave(_sessionJson: string): Promise<{ id: string }> {
    return { id: 'session-mock' };
  },

  async sessionCreate(_title: string): Promise<{ id: string }> {
    return { id: `session-${Date.now()}` };
  },

  async chatSend(_sessionId: string, _content: string, _caseId?: string, _workspaceRoot?: string): Promise<{ turn_id: string; session_id: string }> {
    return { turn_id: 'turn-mock', session_id: 'session-mock' };
  },

  async chatCancel(_sessionId: string): Promise<void> {
    /* mock no-op */
  },

  async permissionRespond(_requestId: string, _outcome: string): Promise<void> {
    /* mock no-op */
  },

  async onStream(
    sessionId: string,
    handler: (event: StreamEvent) => void,
  ): Promise<() => void> {
    void sessionId;
    void handler;
    return () => {};
  },

  async caseList() {
    return [];
  },
  async caseLoad(id: string) {
    return {
      id,
      name: 'Mock Case',
      applicationNumber: '',
      status: 'draft',
      documents: [],
      createdAt: '',
      updatedAt: '',
    };
  },
  async caseSave(caseData: import('./types').PatentCase) {
    return caseData;
  },
  async caseCreate(name: string) {
    return {
      id: `case-${Date.now()}`,
      name,
      applicationNumber: '',
      status: 'draft',
      documents: [],
      createdAt: '',
      updatedAt: '',
    };
  },
  async caseDelete(_id: string) {},
  async getWorkspaceInfo() {
    return { workspaceRoot: '.', yunxiHome: '~/.yunxi' };
  },
  async pickWorkspaceFolder(): Promise<string | null> {
    return null;
  },
  async scanWorkspaceRoots(_paths: string[], _maxDepth?: number) {
    return { projects: [] as import('./types').WorkspaceProjectEntry[] };
  },
  async workspaceWatchStart(_paths: string[]) {},
  async workspaceWatchStop() {},
  async shellSessionStart(_workingDir: string) {
    return 'mock-shell';
  },
  async shellSessionWrite(_sessionId: string, _data: string) {},
  async shellSessionClose(_sessionId: string) {},
  async shellSessionResize(_sessionId: string, _rows: number, _cols: number) {},
  async llmAuthConfigured() {
    return true;
  },
  async saveLlmApiKey(_model: string, _apiKey: string) {
    return { model: 'deepseek-v4-pro' } as import('./types').YunxiSettings;
  },
  async listProjectMaterials(_projectFolder: string, _maxDepth?: number) {
    return [] as import('./types').MaterialFileEntry[];
  },
  async importProjectMaterials(caseId: string, _projectFolder: string, _maxFiles?: number, _maxDepth?: number) {
    const c = await this.caseLoad(caseId);
    return {
      imported: [],
      skipped: [],
      errors: [],
      case: c,
    };
  },
  async shellExec(_workingDir: string, command: string) {
    return {
      stdout: `mock$ ${command}\n(预览模式不执行真实 shell)`,
      stderr: '',
      exitCode: 0,
      durationMs: 1,
    };
  },
  async patentSearch(query: string) {
    return `Mock 检索：${query}`;
  },
  async knowledgeSearch(query: string) {
    return `Mock 知识库：${query}`;
  },
  shellChannel(sessionId: string) {
    return `yunxi://shell/${sessionId}`;
  },
  async onShell(_sessionId: string, _handler: (event: import('./types').ShellEvent) => void) {
    return () => {};
  },
  async onWorkspaceChanged(_handler: () => void) {
    return () => {};
  },
};
