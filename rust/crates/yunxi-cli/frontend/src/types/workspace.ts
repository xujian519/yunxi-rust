import type { ViewType } from '@/data/mockData';

/** 侧栏活动视图（VS Code Activity Bar） */
export type SidebarActivity = 'explorer';

/** 底部面板标签 */
export type BottomPanelTab = 'problems' | 'output' | 'terminal';

export type PanelLogLevel = 'info' | 'warn' | 'error';

export interface PanelLogLine {
  id: string;
  time: string;
  level: PanelLogLevel;
  message: string;
  source?: string;
}

export interface PanelProblem {
  id: string;
  severity: 'error' | 'warn';
  message: string;
  source: string;
}

/** 多根工作区文件夹 */
export interface WorkspaceFolder {
  id: string;
  path: string;
  label: string;
  isPrimary?: boolean;
}

export const WORKSPACE_FOLDERS_STORAGE_KEY = 'yunxi.desktop.workspaceFolders.v1';
export const PANEL_HEIGHT_STORAGE_KEY = 'yunxi.desktop.panelHeight.v1';
export const WORKSPACE_SCAN_DEPTH_STORAGE_KEY = 'yunxi.desktop.workspaceScanDepth.v1';
export const WORKSPACE_ARCHIVED_PATHS_KEY = 'yunxi.desktop.archivedPaths.v1';
export const WORKSPACE_WATCH_ENABLED_KEY = 'yunxi.desktop.workspaceWatch.v1';
export const RIGHT_PANEL_WIDTH_STORAGE_KEY = 'yunxi.desktop.rightPanelWidth.v1';

/** 编辑器标签：案件文档、工具视图或外部文件 */
export type EditorTabKind = 'document' | 'tool' | 'external';
export interface EditorTab {
  id: string;
  title: string;
  view: ViewType;
  kind: EditorTabKind;
  caseId?: string;
  docId?: string;
  filePath?: string;
  fileType?: 'pdf' | 'xlsx' | 'xls' | 'docx' | 'doc' | 'txt' | 'md';
}

export function documentTabId(caseId: string, docId: string): string {
  return `doc:${caseId}:${docId}`;
}

export function toolTabId(view: ViewType): string {
  return `tool:${view}`;
}

/** 案件文档类型 → 中心编辑器视图 */
export function viewFromDocType(docType: string): ViewType {
  switch (docType) {
    case 'claims':
      return 'claims';
    case 'description':
      return 'draft';
    case 'search':
      return 'search';
    case 'drafts':
      return 'compare';
    case 'review':
    case 'oa':
      return 'review';
    case 'drawings':
      return 'draft';
    default:
      return 'claims';
  }
}

export type DocxMode = 'markdown' | 'docx'
export type DocxModeMap = Record<string, DocxMode>
