import type { YunxiSettings } from '@/api/types';
import type { ThemeMode } from '@/context/ThemeProvider';

export interface DesktopGeneral {
  language?: string;
  patentOffice?: string;
  sessionDuration?: string;
  autoSave?: boolean;
  notifications?: boolean;
  soundEffects?: boolean;
}

export interface DesktopAppearance {
  theme?: ThemeMode;
  fontSize?: string;
  editorFont?: string;
  density?: string;
  accentColor?: string;
  animations?: boolean;
}

export interface DesktopEditor {
  tabSize?: string;
  wordWrap?: boolean;
  lineNumbers?: boolean;
  highlightActiveLine?: boolean;
  autoFormat?: boolean;
  autoIndent?: boolean;
  showWhitespace?: boolean;
  defaultView?: string;
}

export interface DesktopModelPrefs {
  temperature?: number;
  maxTokens?: number;
  apiBaseUrl?: string;
  timeout?: number;
}

export interface DesktopCost {
  budgetUsd?: number;
  alertThresholdPercent?: number;
}

export interface DesktopShortcutEntry {
  id: string;
  name: string;
  keys: string[];
  category: string;
}

export interface DesktopMaterialsPrefs {
  /** 导入/列举材料时的递归深度（默认 2，上限 5） */
  scanMaxDepth?: number;
}

export interface DesktopSettingsBlob {
  general?: DesktopGeneral;
  appearance?: DesktopAppearance;
  editor?: DesktopEditor;
  model?: DesktopModelPrefs;
  cost?: DesktopCost;
  shortcuts?: DesktopShortcutEntry[];
  materials?: DesktopMaterialsPrefs;
}

export function getDesktop(settings: YunxiSettings | null | undefined): DesktopSettingsBlob {
  const raw = settings?.desktop;
  if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
    return raw as DesktopSettingsBlob;
  }
  return {};
}

export function defaultYunxiSettings(model = 'deepseek-v4-pro'): YunxiSettings {
  return { model, desktop: {} };
}

export function withDesktopSection<K extends keyof DesktopSettingsBlob>(
  settings: YunxiSettings,
  section: K,
  patch: Partial<NonNullable<DesktopSettingsBlob[K]>>,
): YunxiSettings {
  const desk = getDesktop(settings);
  const prevSection = desk[section] ?? {};
  return {
    ...settings,
    desktop: {
      ...desk,
      [section]: { ...prevSection, ...patch },
    },
  };
}

export function readBudgetUsd(settings: YunxiSettings | null | undefined, fallback = 50): number {
  const n = getDesktop(settings).cost?.budgetUsd;
  return typeof n === 'number' && Number.isFinite(n) && n > 0 ? n : fallback;
}

export function maskApiKey(keys: Record<string, unknown> | undefined): string {
  if (!keys) return '';
  for (const k of ['DEEPSEEK_API_KEY', 'deepseek', 'openai', 'OPENAI_API_KEY']) {
    const v = keys[k];
    if (typeof v === 'string' && v.length > 0) return v;
  }
  const first = Object.values(keys).find((v) => typeof v === 'string' && v.length > 0);
  return typeof first === 'string' ? first : '';
}

export function withApiKey(settings: YunxiSettings, apiKey: string): YunxiSettings {
  const prev =
    settings.api_keys && typeof settings.api_keys === 'object' && !Array.isArray(settings.api_keys)
      ? (settings.api_keys as Record<string, string>)
      : {};
  const trimmed = apiKey.trim();
  const nextKeys = { ...prev };
  if (trimmed) {
    nextKeys.DEEPSEEK_API_KEY = trimmed;
    nextKeys.deepseek = trimmed;
  } else {
    delete nextKeys.DEEPSEEK_API_KEY;
    delete nextKeys.deepseek;
  }
  return { ...settings, api_keys: nextKeys };
}

export type PermissionDefaultMode = 'dontAsk' | 'plan' | 'read-only' | 'workspace-write' | 'danger-full-access';

export function readPermissionMode(settings: YunxiSettings | null | undefined): PermissionDefaultMode {
  const perms = settings?.permissions;
  if (perms && typeof perms === 'object' && !Array.isArray(perms)) {
    const mode = (perms as Record<string, unknown>).defaultMode;
    if (typeof mode === 'string') return mode as PermissionDefaultMode;
  }
  return 'dontAsk';
}

export function withPermissionMode(
  settings: YunxiSettings,
  defaultMode: PermissionDefaultMode,
): YunxiSettings {
  return {
    ...settings,
    permissions: { defaultMode },
  };
}
