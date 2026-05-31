import type { ShortcutRecord } from '@/utils/shortcutDefaults';

const MODIFIER_TOKENS = new Set(['⌘', '⇧', '⌥', '⌃']);

function eventKeyChar(e: KeyboardEvent): string {
  if (e.key === 'Enter') return '↵';
  if (e.key === '/') return '/';
  if (e.key === '`') return '`';
  if (e.key === ',') return ',';
  if (e.key.length === 1) return e.key.toUpperCase();
  return e.key;
}

function modifiersMatch(e: KeyboardEvent, keys: string[]): boolean {
  const wantMeta = keys.includes('⌘');
  const wantShift = keys.includes('⇧');
  const wantAlt = keys.includes('⌥');
  const wantCtrl = keys.includes('⌃');
  const isMac = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform);

  if (wantMeta) {
    if (isMac ? !e.metaKey : !e.ctrlKey) return false;
  } else if (e.metaKey || e.ctrlKey) {
    return false;
  }
  if (wantShift !== e.shiftKey) return false;
  if (wantAlt !== e.altKey) return false;
  if (wantCtrl && isMac && !e.ctrlKey) return false;
  return true;
}

/** 判断键盘事件是否匹配快捷键定义 */
export function matchShortcut(e: KeyboardEvent, keys: string[]): boolean {
  const mainTokens = keys.filter((k) => !MODIFIER_TOKENS.has(k));
  if (mainTokens.length === 0) return false;
  const expected = mainTokens[mainTokens.length - 1];
  if (eventKeyChar(e) !== expected && e.key !== expected) {
    if (expected === '↵' && e.key !== 'Enter') return false;
    if (expected !== eventKeyChar(e)) return false;
  }
  return modifiersMatch(e, keys);
}

export function findMatchingShortcut(
  e: KeyboardEvent,
  shortcuts: ShortcutRecord[],
): ShortcutRecord | undefined {
  return shortcuts.find((s) => matchShortcut(e, s.keys));
}

/** 是否在可编辑元素中（避免覆盖输入） */
export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return target.isContentEditable;
}
