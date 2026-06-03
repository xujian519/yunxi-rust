/** IPC 流式事件（与 Rust StreamEvent 对齐） */
export type StreamEvent =
  | { type: 'text_delta'; content: string }
  | { type: 'reasoning_delta'; content: string }
  | { type: 'tool_use'; id: string; name: string; input: string }
  | { type: 'tool_result'; id: string; output: string; is_error: boolean }
  | { type: 'permission_request'; request_id: string; tool: string; input: string }
  | { type: 'usage'; input_tokens: number; output_tokens: number }
  | { type: 'message_stop' }
  | { type: 'error'; message: string };

export interface UsageSummary {
  input_tokens: number;
  output_tokens: number;
  estimated_cost: number;
}

export interface YunxiSettings {
  model: string;
  model_router?: Record<string, unknown>;
  permissions?: { defaultMode: string };
  hooks?: {
    PreToolUse?: string[];
    PostToolUse?: string[];
  };
  appearance?: Record<string, unknown>;
  api_keys?: Record<string, string>;
  /** 桌面端 UI 偏好分区 */
  desktop?: Record<string, unknown>;
}

export interface SessionMeta {
  id: string;
  message_count: number;
  modified_at: number;
}

export interface ChatSendResult {
  turn_id: string;
  session_id: string;
}

export interface SessionCreateResult {
  id: string;
}

export interface SessionSaveResult {
  id: string;
}

export interface PatentCaseDocument {
  id: string;
  type: string;
  title: string;
  contentMd: string;
  updatedAt: string;
}

export interface PatentCase {
  id: string;
  name: string;
  applicationNumber: string;
  status: string;
  documents: PatentCaseDocument[];
  activeSessionId?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface WorkspaceInfo {
  workspaceRoot: string;
  yunxiHome: string;
}

export interface WorkspaceProjectEntry {
  folderPath: string;
  label: string;
  isPatentProject: boolean;
  caseId?: string | null;
  caseName?: string | null;
  workspaceRoot: string;
}

export interface ScanWorkspaceResult {
  projects: WorkspaceProjectEntry[];
}

export interface ShellExecResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  durationMs: number;
}

export type ShellEvent =
  | { type: 'output'; data: string }
  | { type: 'exit'; code: number | null }
  | { type: 'error'; message: string };

export interface MaterialFileEntry {
  path: string;
  name: string;
  extension: string;
  sizeBytes: number;
}

export interface ImportMaterialsResult {
  imported: string[];
  skipped: string[];
  errors: string[];
  case: PatentCase;
}

export interface DirectoryEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
}

/** 斜杠命令执行结果（与 Rust SlashExecuteResult 对齐） */
export type SlashExecuteResult =
  | { kind: 'message'; content: string }
  | { kind: 'agent_turn'; prompt: string }
  | { kind: 'session_updated'; content: string; session_json: string };

export interface DoctorCheck {
  name: string;
  status: string;
  detail: string;
}

export interface DoctorReport {
  checks: DoctorCheck[];
  failures: number;
  warnings: number;
  summary: string;
}

export interface McpServerStatus {
  name: string;
  transport: string;
  status: string;
  tool_count: number;
  detail?: string | null;
}

export interface McpStatusReport {
  servers: McpServerStatus[];
  total_tools: number;
}

export interface OAuthStatus {
  configured: boolean;
}
