import { httpChatSend, httpOnStream, httpPermissionRespond } from './httpChat';
import type {
  DoctorReport,
  McpStatusReport,
  OAuthStatus,
  PatentCase,
  SessionMeta,
  SlashExecuteResult,
  StreamEvent,
  UsageSummary,
  YunxiSettings,
  WorkspaceInfo,
} from './types';

const serverUrl = (): string => {
  const base = import.meta.env.VITE_SERVER_URL as string | undefined;
  return (base ?? 'http://127.0.0.1:8765').replace(/\/$/, '');
};

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${serverUrl()}${path}`);
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

async function postJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${serverUrl()}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

async function deleteReq(path: string): Promise<void> {
  const res = await fetch(`${serverUrl()}${path}`, { method: 'DELETE' });
  if (!res.ok && res.status !== 204) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `HTTP ${res.status}`);
  }
}

async function putJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${serverUrl()}${path}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

async function getText(path: string): Promise<string> {
  const res = await fetch(`${serverUrl()}${path}`);
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `HTTP ${res.status}`);
  }
  return res.text();
}

async function putEmpty(path: string, body: unknown): Promise<void> {
  const res = await fetch(`${serverUrl()}${path}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok && res.status !== 204) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(text || `HTTP ${res.status}`);
  }
}

async function executeTool(name: string, input: Record<string, unknown>): Promise<string> {
  const data = await postJson<{ result: string }>('/api/tools/execute', { name, input });
  return data.result;
}

function formatKnowledgeResults(json: {
  total?: number;
  results?: Array<{ title?: string; content?: string; source?: string }>;
}): string {
  const rows = json.results ?? [];
  if (rows.length === 0) return '无检索结果';
  return rows
    .slice(0, 8)
    .map((r, i) => {
      const title = r.title ?? r.source ?? '条目';
      const body = (r.content ?? '').slice(0, 400);
      return `${i + 1}. ${title}\n${body}`;
    })
    .join('\n\n');
}

/** HTTP Server API — 浏览器预览模式对接 yunxi-server */
export const httpApi = {
  async getVersion(): Promise<string> {
    await getJson<{ status: string }>('/api/health');
    return 'yunxi-server';
  },

  async getSettings(): Promise<YunxiSettings> {
    return getJson<YunxiSettings>('/api/settings');
  },

  async saveSettings(settings: YunxiSettings): Promise<void> {
    await putEmpty('/api/settings', settings);
  },

  async getUsage(): Promise<UsageSummary> {
    return { input_tokens: 0, output_tokens: 0, estimated_cost: 0 };
  },

  async sessionList(): Promise<SessionMeta[]> {
    return getJson<SessionMeta[]>('/api/sessions');
  },

  async sessionLoad(id: string): Promise<string> {
    return getText(`/api/sessions/${encodeURIComponent(id)}`);
  },

  async sessionSave(sessionJson: string): Promise<{ id: string }> {
    let id = 'default';
    try {
      const parsed = JSON.parse(sessionJson) as { id?: string };
      if (parsed.id) id = parsed.id;
    } catch {
      /* use default */
    }
    return putJson<{ id: string }>(`/api/sessions/${encodeURIComponent(id)}`, {
      session_json: sessionJson,
    });
  },

  async sessionCreate(title: string): Promise<{ id: string }> {
    return postJson<{ id: string }>('/api/sessions', { title });
  },

  async sessionDelete(id: string): Promise<void> {
    await deleteReq(`/api/sessions/${encodeURIComponent(id)}`);
  },

  async chatSend(
    sessionId: string,
    content: string,
    _caseId?: string,
    _workspaceRoot?: string,
  ): Promise<{ turn_id: string; session_id: string }> {
    const settings = await this.getSettings();
    return httpChatSend(sessionId, content, settings.model);
  },

  async chatCancel(_sessionId: string): Promise<void> {},

  async permissionRespond(requestId: string, outcome: string): Promise<void> {
    httpPermissionRespond(requestId, outcome);
    const res = await fetch(`${serverUrl()}/api/chat/permission`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ request_id: requestId, outcome }),
    });
    if (!res.ok && res.status !== 204 && res.status !== 404) {
      const text = await res.text().catch(() => res.statusText);
      throw new Error(text || `HTTP ${res.status}`);
    }
  },

  async onStream(sessionId: string, handler: (event: StreamEvent) => void): Promise<() => void> {
    return httpOnStream(sessionId, handler);
  },

  async caseList(): Promise<PatentCase[]> {
    return getJson<PatentCase[]>('/api/cases');
  },

  async caseLoad(id: string): Promise<PatentCase> {
    return getJson<PatentCase>(`/api/cases/${encodeURIComponent(id)}`);
  },

  async caseSave(caseData: PatentCase): Promise<PatentCase> {
    return putJson<PatentCase>(`/api/cases/${encodeURIComponent(caseData.id)}`, caseData);
  },

  async caseCreate(name: string, applicationNumber?: string): Promise<PatentCase> {
    return postJson<PatentCase>('/api/cases', { name, application_number: applicationNumber });
  },

  async caseDelete(id: string): Promise<void> {
    await deleteReq(`/api/cases/${encodeURIComponent(id)}`);
  },

  async getWorkspaceInfo(): Promise<WorkspaceInfo> {
    return { workspaceRoot: '.', yunxiHome: '~/.yunxi' };
  },

  async pickWorkspaceFolder(): Promise<string | null> {
    return null;
  },

  async scanWorkspaceRoots(_paths: string[], _maxDepth?: number) {
    return { projects: [] };
  },

  async workspaceWatchStart(_paths: string[]): Promise<void> {},
  async workspaceWatchStop(): Promise<void> {},

  async shellSessionStart(_workingDir: string): Promise<string> {
    return 'http-no-shell';
  },
  async shellSessionWrite(_sessionId: string, _data: string): Promise<void> {},
  async shellSessionClose(_sessionId: string): Promise<void> {},
  async shellSessionResize(_sessionId: string, _rows: number, _cols: number): Promise<void> {},

  async llmAuthConfigured(): Promise<boolean> {
    return true;
  },

  async saveLlmApiKey(_model: string, _apiKey: string): Promise<YunxiSettings> {
    return { model: 'claude-sonnet-4-20250514', permissions: { defaultMode: 'dontAsk' } };
  },

  async listProjectMaterials(_projectFolder: string, _maxDepth?: number) {
    return [];
  },

  async importProjectMaterials(caseId: string, _projectFolder: string, _maxFiles?: number, _maxDepth?: number) {
    const c = await this.caseLoad(caseId);
    return { imported: [], skipped: [], errors: [], case: c };
  },

  async shellExec(_workingDir: string, command: string) {
    return {
      stdout: `server$ ${command}\n(HTTP 模式不支持 shell)`,
      stderr: '',
      exitCode: 0,
      durationMs: 0,
    };
  },

  async patentSearch(query: string, limit?: number) {
    return executeTool('PatentSearch', { query, limit: limit ?? 10 });
  },

  async knowledgeSearch(query: string) {
    const json = await getJson<{ total: number; results: unknown[] }>(
      `/api/knowledge/search?q=${encodeURIComponent(query)}&limit=8`,
    );
    return formatKnowledgeResults(json as Parameters<typeof formatKnowledgeResults>[0]);
  },

  async memorySearch(query: string, limit?: number) {
    const json = await getJson<{ count: number; entries: Array<{ content: string }> }>(
      `/api/memory/search?q=${encodeURIComponent(query)}&limit=${limit ?? 10}`,
    );
    if (json.count === 0) return `Memory search\n  Query   ${query}\n  Results 0`;
    return json.entries.map((e, i) => `${i + 1}. ${e.content.slice(0, 300)}`).join('\n\n');
  },

  async oaParse(content: string, applicationNumber?: string) {
    return executeTool('OaParse', {
      content,
      application_number: applicationNumber,
      document_type: 'cn',
    });
  },

  async patentCompare(
    targetTitle: string,
    targetClaims: string[],
    priorTitle: string,
    priorClaims: string[],
  ) {
    return executeTool('PatentCompare', {
      mode: 'diff',
      target: { title: targetTitle, claims: targetClaims },
      priorArt: { title: priorTitle, claims: priorClaims },
    });
  },

  async oauthStatus(): Promise<OAuthStatus> {
    return { configured: false };
  },
  async oauthLogin(): Promise<void> {},
  async oauthLogout(): Promise<void> {},

  async runDoctorCheck(): Promise<DoctorReport> {
    return { summary: 'HTTP 模式未接 doctor', checks: [], failures: 0, warnings: 0 };
  },

  async initWorkspace(): Promise<string> {
    return 'HTTP 模式请使用 CLI init';
  },

  async initClaudeMd(): Promise<string> {
    return 'HTTP 模式请使用 CLI';
  },

  async getMcpStatus(): Promise<McpStatusReport> {
    return getJson<McpStatusReport>('/api/mcp/status');
  },

  async getMcpConfig(): Promise<Record<string, unknown>> {
    const status = await this.getMcpStatus();
    const out: Record<string, unknown> = {};
    for (const s of status.servers) {
      out[s.name] = { transport: s.transport, status: s.status };
    }
    return out;
  },

  async executeSlashCommand(
    _sessionId: string,
    input: string,
    _model?: string,
    _workspaceRoot?: string,
  ): Promise<SlashExecuteResult | null> {
    if (!input.trim().startsWith('/')) return null;
    return {
      kind: 'message',
      content: 'HTTP 模式完整斜杠命令请使用桌面客户端；/search /analyze 可在聊天输入框使用。',
    };
  },

  shellChannel(sessionId: string): string {
    return `yunxi://shell/${sessionId}`;
  },

  async onShell(_sessionId: string, _handler: (event: import('./types').ShellEvent) => void) {
    return () => {};
  },

  async onWorkspaceChanged(_handler: () => void) {
    return () => {};
  },
};
