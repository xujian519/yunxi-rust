/** 可序列化快捷键项（写入 settings.desktop.shortcuts） */
export interface ShortcutRecord {
  id: string;
  name: string;
  keys: string[];
  category: string;
}

export const DEFAULT_SHORTCUT_RECORDS: ShortcutRecord[] = [
  { id: 'new-session', name: '新建会话', keys: ['⌘', 'N'], category: '应用' },
  { id: 'toggle-sidebar', name: '切换侧栏', keys: ['⌘', 'B'], category: '导航' },
  { id: 'toggle-ai-panel', name: '切换 AI 面板', keys: ['⌘', 'J'], category: '导航' },
  { id: 'toggle-terminal', name: '切换底栏终端', keys: ['⌘', '`'], category: '导航' },
  { id: 'toggle-theme', name: '切换主题', keys: ['⌘', '/'], category: '应用' },
  { id: 'open-settings', name: '打开设置', keys: ['⌘', ','], category: '应用' },
  { id: 'focus-search', name: '资源管理器搜索', keys: ['⌘', 'K'], category: '应用' },
  { id: 'command-palette', name: '命令面板', keys: ['⇧', '⌘', 'P'], category: '应用' },
];

export function shortcutsFromSettings(
  raw: unknown[] | undefined,
): ShortcutRecord[] {
  if (!Array.isArray(raw) || raw.length === 0) {
    return DEFAULT_SHORTCUT_RECORDS;
  }
  const parsed: ShortcutRecord[] = [];
  for (const item of raw) {
    if (!item || typeof item !== 'object') continue;
    const o = item as Record<string, unknown>;
    if (typeof o.id !== 'string' || typeof o.name !== 'string') continue;
    const keys = Array.isArray(o.keys)
      ? o.keys.filter((k): k is string => typeof k === 'string')
      : [];
    if (keys.length === 0) continue;
    parsed.push({
      id: o.id,
      name: o.name,
      keys,
      category: typeof o.category === 'string' ? o.category : '应用',
    });
  }
  if (parsed.length === 0) return DEFAULT_SHORTCUT_RECORDS;
  return mergeShortcutDefaults(parsed);
}

/** 将 settings 中缺失的默认快捷键补全（如新增的 command-palette） */
export function mergeShortcutDefaults(saved: ShortcutRecord[]): ShortcutRecord[] {
  const byId = new Map(saved.map((s) => [s.id, s]));
  const merged: ShortcutRecord[] = [];
  for (const def of DEFAULT_SHORTCUT_RECORDS) {
    merged.push(byId.get(def.id) ?? def);
    byId.delete(def.id);
  }
  for (const extra of byId.values()) merged.push(extra);
  return merged;
}
