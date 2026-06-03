import { mockApi } from './mock';
import { httpApi } from './http';
import { tauriApi } from './tauri';

const useMock = import.meta.env.VITE_USE_MOCK === 'true';
const serverUrl = import.meta.env.VITE_SERVER_URL as string | undefined;

function hasTauri(): boolean {
  if (typeof window === 'undefined') return false;
  const t = window.__TAURI__;
  return Boolean(t?.core?.invoke ?? t?.invoke);
}

function resolveApi() {
  if (useMock) return mockApi;
  if (hasTauri()) return tauriApi;
  if (serverUrl) return httpApi;
  return mockApi;
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
  return hasTauri() && !useMock && !serverUrl;
}

/** 是否通过 HTTP Server 对接后端（浏览器 + VITE_SERVER_URL） */
export function isHttpServerRuntime(): boolean {
  return !hasTauri() && !useMock && Boolean(serverUrl);
}

/** 是否可调用 Agent 对话（Tauri 或 HTTP Server） */
export function isAgentBackend(): boolean {
  return isTauriRuntime() || isHttpServerRuntime();
}

/** 是否可调用 Rust 工具 API（Tauri IPC 或 HTTP Server） */
export function hasBackendTools(): boolean {
  return isAgentBackend();
}

export type {
  ChatSendResult,
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
  WorkspaceProjectEntry,
  ScanWorkspaceResult,
  ShellExecResult,
} from './types';
