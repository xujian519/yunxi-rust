import { useEffect } from 'react';
import type { FC } from 'react';
import { useNavigate } from 'react-router';
import { useApp } from '@/context/AppProvider';
import { useTheme } from '@/context/ThemeProvider';
import { getDesktop } from '@/utils/desktopSettings';
import {
  DEFAULT_SHORTCUT_RECORDS,
  shortcutsFromSettings,
} from '@/utils/shortcutDefaults';
import { findMatchingShortcut, isEditableTarget } from '@/utils/keyboardShortcuts';

export interface DesktopShortcutActions {
  onToggleSidebar: () => void;
  onToggleAiPanel: () => void;
}

interface DesktopShortcutHandlerProps extends DesktopShortcutActions {}

const DesktopShortcutHandler: FC<DesktopShortcutHandlerProps> = ({
  onToggleSidebar,
  onToggleAiPanel,
}) => {
  const navigate = useNavigate();
  const { toggle, resolved: _resolved } = useTheme();
  const {
    yunxiSettings,
    createSession,
    toggleBottomPanel,
    setCommandPaletteOpen,
  } = useApp();

  useEffect(() => {
    const raw = getDesktop(yunxiSettings).shortcuts as unknown[] | undefined;
    const shortcuts = shortcutsFromSettings(raw);

    const onKeyDown = (e: KeyboardEvent) => {
      if (isEditableTarget(e.target)) return;
      const hit = findMatchingShortcut(e, shortcuts);
      if (!hit) return;

      switch (hit.id) {
        case 'new-session':
          e.preventDefault();
          void createSession();
          break;
        case 'toggle-sidebar':
          e.preventDefault();
          onToggleSidebar();
          break;
        case 'toggle-ai-panel':
          e.preventDefault();
          onToggleAiPanel();
          break;
        case 'toggle-terminal':
          e.preventDefault();
          toggleBottomPanel('terminal');
          break;
        case 'toggle-theme':
          e.preventDefault();
          toggle();
          break;
        case 'open-settings':
          e.preventDefault();
          navigate('/settings');
          break;
        case 'focus-search':
          e.preventDefault();
          document.querySelector<HTMLInputElement>('[data-explorer-search]')?.focus();
          break;
        case 'command-palette':
          e.preventDefault();
          setCommandPaletteOpen(true);
          break;
        default:
          break;
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
    yunxiSettings,
    createSession,
    toggleBottomPanel,
    setCommandPaletteOpen,
    onToggleSidebar,
    onToggleAiPanel,
    toggle,
    navigate,
  ]);

  return null;
};

export { DEFAULT_SHORTCUT_RECORDS };
export default DesktopShortcutHandler;
