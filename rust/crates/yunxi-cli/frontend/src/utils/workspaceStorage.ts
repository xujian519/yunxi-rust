import type { WorkspaceFolder } from '@/types/workspace';
import {
  WORKSPACE_FOLDERS_STORAGE_KEY,
  PANEL_HEIGHT_STORAGE_KEY,
  WORKSPACE_SCAN_DEPTH_STORAGE_KEY,
  WORKSPACE_ARCHIVED_PATHS_KEY,
  WORKSPACE_WATCH_ENABLED_KEY,
  RIGHT_PANEL_WIDTH_STORAGE_KEY,
} from '@/types/workspace';

export function loadWorkspaceFolders(): WorkspaceFolder[] {
  try {
    const raw = localStorage.getItem(WORKSPACE_FOLDERS_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as WorkspaceFolder[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function saveWorkspaceFolders(folders: WorkspaceFolder[]): void {
  localStorage.setItem(WORKSPACE_FOLDERS_STORAGE_KEY, JSON.stringify(folders));
}

export function loadPanelHeight(defaultHeight = 180): number {
  try {
    const raw = localStorage.getItem(PANEL_HEIGHT_STORAGE_KEY);
    if (!raw) return defaultHeight;
    const n = Number(raw);
    return Number.isFinite(n) && n >= 80 && n <= 480 ? n : defaultHeight;
  } catch {
    return defaultHeight;
  }
}

export function savePanelHeight(height: number): void {
  localStorage.setItem(PANEL_HEIGHT_STORAGE_KEY, String(Math.round(height)));
}

export function folderLabelFromPath(path: string): string {
  const trimmed = path.replace(/\/$/, '');
  const parts = trimmed.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

export function newFolderId(): string {
  return `wf-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

/** 工作区 YUNXI.md 扫描深度（1–5，默认 2） */
export function loadWorkspaceScanDepth(defaultDepth = 2): number {
  try {
    const raw = localStorage.getItem(WORKSPACE_SCAN_DEPTH_STORAGE_KEY);
    if (!raw) return defaultDepth;
    const n = Number(raw);
    return Number.isFinite(n) && n >= 1 && n <= 5 ? Math.round(n) : defaultDepth;
  } catch {
    return defaultDepth;
  }
}

export function saveWorkspaceScanDepth(depth: number): void {
  const clamped = Math.min(5, Math.max(1, Math.round(depth)));
  localStorage.setItem(WORKSPACE_SCAN_DEPTH_STORAGE_KEY, String(clamped));
}

export function loadArchivedPaths(): string[] {
  try {
    const raw = localStorage.getItem(WORKSPACE_ARCHIVED_PATHS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as string[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function saveArchivedPaths(paths: string[]): void {
  localStorage.setItem(WORKSPACE_ARCHIVED_PATHS_KEY, JSON.stringify(paths));
}

/** 默认关闭：避免开发时 target/会话写入触发整页重绘 */
export function loadWorkspaceWatchEnabled(defaultEnabled = false): boolean {
  try {
    const raw = localStorage.getItem(WORKSPACE_WATCH_ENABLED_KEY);
    if (raw === null) return defaultEnabled;
    return raw === '1' || raw === 'true';
  } catch {
    return defaultEnabled;
  }
}

export function saveWorkspaceWatchEnabled(enabled: boolean): void {
  localStorage.setItem(WORKSPACE_WATCH_ENABLED_KEY, enabled ? '1' : '0');
}

/** 右侧 AI 面板宽度（320–560，默认 380） */
export function loadRightPanelWidth(defaultWidth = 380): number {
  try {
    const raw = localStorage.getItem(RIGHT_PANEL_WIDTH_STORAGE_KEY);
    if (!raw) return defaultWidth;
    const n = Number(raw);
    return Number.isFinite(n) && n >= 320 && n <= 560 ? Math.round(n) : defaultWidth;
  } catch {
    return defaultWidth;
  }
}

export function saveRightPanelWidth(width: number): void {
  const clamped = Math.min(560, Math.max(320, Math.round(width)));
  localStorage.setItem(RIGHT_PANEL_WIDTH_STORAGE_KEY, String(clamped));
}
