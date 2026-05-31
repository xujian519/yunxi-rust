const STORAGE_KEY = 'yunxi-palette-recent';
const MAX_RECENT = 8;

export function getPaletteRecent(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((id): id is string => typeof id === 'string').slice(0, MAX_RECENT);
  } catch {
    return [];
  }
}

export function recordPaletteRecent(commandId: string): void {
  const prev = getPaletteRecent().filter((id) => id !== commandId);
  const next = [commandId, ...prev].slice(0, MAX_RECENT);
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  } catch {
    /* ignore quota */
  }
}
