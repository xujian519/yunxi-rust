import { mockApi } from './mock';
import { tauriApi } from './tauri';

const useMock = import.meta.env.VITE_USE_MOCK === 'true';

function hasTauri(): boolean {
  if (typeof window === 'undefined') return false;
  const t = window.__TAURI__;
  return Boolean(t?.core?.invoke ?? t?.invoke);
}

function resolveApi() {
  return useMock || !hasTauri() ? mockApi : tauriApi;
}

/** 统一 API — 每次访问时检测 Tauri，避免模块加载过早导致误判 */
export const api = new Proxy({} as typeof tauriApi, {
  get(_target, prop) {
    const impl = resolveApi();
    const value = impl[prop as keyof typeof impl];
    if (typeof value === 'function') {
      return (value as (...args: unknown[]) => unknown).bind(impl);
    }
    return value;
  },
});

export function isTauriRuntime(): boolean {
  return hasTauri() && !useMock;
}

export type {
  ChatSendResult,
  PatentCase,
  SessionMeta,
  StreamEvent,
  UsageSummary,
  YunxiSettings,
  WorkspaceInfo,
  WorkspaceProjectEntry,
  ScanWorkspaceResult,
  ShellExecResult,
} from './types';
