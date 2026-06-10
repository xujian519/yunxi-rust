import type {
  ChatSendResult,
  DirectoryEntry,
  DoctorReport,
  ImportMaterialsResult,
  MaterialFileEntry,
  McpStatusReport,
  OAuthStatus,
  PatentCase,
  SessionCreateResult,
  SessionMeta,
  SessionSaveResult,
  SlashExecuteResult,
  StreamEvent,
  UsageSummary,
  WorkspaceInfo,
  ScanWorkspaceResult,
  ShellExecResult,
  ShellEvent,
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
  sessionDelete: (id: string) => invoke<void>('session_delete', { id }),

  chatSend: (sessionId: string, content: string, caseId?: string, workspaceRoot?: string) =>
    invoke<ChatSendResult>('chat_send', { sessionId, content, caseId, workspaceRoot }),
  chatCancel: (sessionId: string) => invoke<void>('chat_cancel', { sessionId }),

  permissionRespond: (requestId: string, outcome: 'allow' | 'deny' | 'always') =>
    invoke<void>('permission_respond', { requestId, outcome }),

  getWorkspaceInfo: () => invoke<WorkspaceInfo>('get_workspace_info'),
  pickWorkspaceFolder: () => invoke<string | null>('pick_workspace_folder'),
  scanWorkspaceRoots: (paths: string[], maxDepth?: number) =>
    invoke<ScanWorkspaceResult>('scan_workspace_roots', { paths, maxDepth }),
  listDirectory: (dir: string) => invoke<DirectoryEntry[]>('list_directory', { dir }),
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
  claimParse: (claims: string) =>
    invoke<string>('claim_parse', { claims }),
  noveltyAnalysis: (claims: string, priorArt: string, analysisMode?: string) =>
    invoke<string>('novelty_analysis', {
      claims,
      prior_art: priorArt,
      analysis_mode: analysisMode,
    }),
  inventivenessAnalysis: (claims: string, priorArt: string, technicalField?: string) =>
    invoke<string>('inventiveness_analysis', {
      claims,
      prior_art: priorArt,
      technical_field: technicalField,
    }),
  claimGenerator: (technicalFeatures: string, claimType?: string, scope?: string) =>
    invoke<string>('claim_generator', {
      technical_features: technicalFeatures,
      claim_type: claimType,
      scope,
    }),
  abstractDrafter: (
    inventionTitle: string,
    technicalField: string,
    technicalProblem: string,
    technicalSolution: string,
    beneficialEffects?: string,
  ) =>
    invoke<string>('abstract_drafter', {
      invention_title: inventionTitle,
      technical_field: technicalField,
      technical_problem: technicalProblem,
      technical_solution: technicalSolution,
      beneficial_effects: beneficialEffects,
    }),
  specificationDrafter: (
    claims: string,
    abstractText: string,
    technicalField: string,
    background?: string,
    detailedDescription?: string,
  ) =>
    invoke<string>('specification_drafter', {
      claims,
      abstract: abstractText,
      technical_field: technicalField,
      background,
      detailed_description: detailedDescription,
    }),
  qualityScorer: (
    claims: string,
    abstractText?: string,
    specification?: string,
    scoreMode?: string,
  ) =>
    invoke<string>('quality_scorer', {
      claims,
      abstract: abstractText,
      specification,
      score_mode: scoreMode,
    }),
  qualityChecker: (
    claims: string,
    abstractText?: string,
    specification?: string,
    checkMode?: string,
  ) =>
    invoke<string>('quality_checker', {
      claims,
      abstract: abstractText,
      specification,
      check_mode: checkMode,
    }),
  formalCheck: (patentText: string, checkType?: string, jurisdiction?: string) =>
    invoke<string>('formal_check', {
      patent_text: patentText,
      check_type: checkType,
      jurisdiction,
    }),
  claimFormalityCheck: (claims: string, jurisdiction?: string) =>
    invoke<string>('claim_formality_check', {
      claims,
      jurisdiction,
    }),
  specFormalityCheck: (specification: string, jurisdiction?: string) =>
    invoke<string>('spec_formality_check', {
      specification,
      jurisdiction,
    }),
  oaStrategy: (
    claims: string,
    priorArt: string,
    rejectionReasons: string,
    jurisdiction?: string,
  ) =>
    invoke<string>('oa_strategy', {
      claims,
      prior_art: priorArt,
      rejection_reasons: rejectionReasons,
      jurisdiction,
    }),
  responseTemplate: (
    oaType: string,
    rejectionReasons: string,
    strategy?: string,
    jurisdiction?: string,
  ) =>
    invoke<string>('response_template', {
      oa_type: oaType,
      rejection_reasons: rejectionReasons,
      strategy,
      jurisdiction,
    }),
  successPredictor: (
    claims: string,
    priorArt: string,
    technicalField: string,
    jurisdiction?: string,
  ) =>
    invoke<string>('success_predictor', {
      claims,
      prior_art: priorArt,
      technical_field: technicalField,
      jurisdiction,
    }),
  infringementAnalysis: (
    patentClaims: string,
    productFeatures: string,
    analysisMode?: string,
  ) =>
    invoke<string>('infringement_analysis', {
      patent_claims: patentClaims,
      product_features: productFeatures,
      analysis_mode: analysisMode,
    }),
  legalReasoning: (legalQuestion: string, jurisdiction?: string, context?: string) =>
    invoke<string>('legal_reasoning', {
      legal_question: legalQuestion,
      jurisdiction,
      context,
    }),
  examinerSimulate: (
    claims: string,
    priorArt: string,
    technicalField: string,
    simulateMode?: string,
  ) =>
    invoke<string>('examiner_simulate', {
      claims,
      prior_art: priorArt,
      technical_field: technicalField,
      simulate_mode: simulateMode,
    }),
  hybridRetrieval: (
    query: string,
    vectorWeight?: number,
    graphWeight?: number,
    legalWeight?: number,
    topK?: number,
  ) =>
    invoke<string>('hybrid_retrieval', {
      query,
      vector_weight: vectorWeight,
      graph_weight: graphWeight,
      legal_weight: legalWeight,
      top_k: topK,
    }),
  knowledgeSearch: (query: string) => invoke<string>('knowledge_search', { query }),
  memorySearch: (query: string, limit?: number) =>
    invoke<string>('memory_search', { query, limit }),
  recordIntentPreference: (intentType: string) =>
    invoke<void>('record_intent_preference', { intentType }),


  lawQuery: (query: string) => invoke<string>('law_query', { query }),
  knowledgeCard: (topic: string) => invoke<string>('knowledge_card', { topic }),
  superReasoningPlan: (query: string) =>
    invoke<string>('super_reasoning_plan', { query }),
  innovationEvaluator: (
    inventionTitle: string,
    technicalField: string,
    technicalProblem: string,
    technicalSolution: string,
  ) =>
    invoke<string>('innovation_evaluator', {
      inventionTitle,
      technicalField,
      technicalProblem,
      technicalSolution,
    }),
  semanticCompare: (
    targetText: string,
    priorText: string,
    mode?: string,
  ) =>
    invoke<string>('semantic_compare', {
      targetText,
      priorText,
      mode,
    }),

  oaParse: (content: string, applicationNumber?: string) =>
    invoke<string>('oa_parse', { content, applicationNumber }),

  patentCompare: (
    targetTitle: string,
    targetClaims: string[],
    priorTitle: string,
    priorClaims: string[],
  ) =>
    invoke<string>('patent_compare', {
      targetTitle,
      targetClaims,
      priorTitle,
      priorClaims,
    }),

  oauthStatus: () => invoke<OAuthStatus>('oauth_status'),
  oauthLogin: () => invoke<void>('oauth_login'),
  oauthLogout: () => invoke<void>('oauth_logout'),
  runDoctorCheck: () => invoke<DoctorReport>('run_doctor_check'),
  initWorkspace: () => invoke<string>('init_workspace'),
  initClaudeMd: () => invoke<string>('init_claude_md'),
  getMcpStatus: () => invoke<McpStatusReport>('get_mcp_status'),
  getMcpConfig: () => invoke<Record<string, unknown>>('get_mcp_config'),
  executeSlashCommand: (
    sessionId: string,
    input: string,
    model?: string,
    workspaceRoot?: string,
  ) =>
    invoke<SlashExecuteResult | null>('execute_slash_command', {
      sessionId,
      input,
      model,
      workspaceRootArg: workspaceRoot,
    }),

  executeToolRaw: (toolName: string, toolInput: Record<string, unknown>) =>
    invoke<string>('execute_tool_raw', { toolName, toolInput }),

  runReasoning: (
    query: string,
    context?: string,
    phases?: string[],
    config?: Record<string, unknown>,
  ) =>
    invoke<string>('run_reasoning', {
      query,
      context,
      phases,
      config,
    }),

  listReasoningPhases: () => invoke<string[]>('list_reasoning_phases'),

  getPipelineConfig: () => invoke<Record<string, unknown>>('get_pipeline_config'),

  checkCompliance: (text: string, ruleTypes?: string[], context?: string) =>
    invoke<string>('check_compliance', {
      text,
      ruleTypes,
      context,
    }),

  listRuleTypes: () => invoke<string[]>('list_rule_types'),

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
