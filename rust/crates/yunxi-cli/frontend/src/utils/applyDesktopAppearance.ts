import type { YunxiSettings } from '@/api/types';
import type { ResolvedTheme } from '@/context/ThemeProvider';
import { getDesktop } from '@/utils/desktopSettings';

type AccentId = 'sage' | 'blue' | 'purple' | 'orange';

interface AccentPalette {
  primary: string;
  hover: string;
  muted: string;
  cyan: string;
}

const ACCENTS: Record<AccentId, { light: AccentPalette; dark: AccentPalette }> = {
  sage: {
    light: {
      primary: '#4A7C6F',
      hover: '#3D6A5E',
      muted: 'rgba(74, 124, 111, 0.12)',
      cyan: '#3A8B8C',
    },
    dark: {
      primary: '#5FA08F',
      hover: '#7BB8A8',
      muted: 'rgba(95, 160, 143, 0.15)',
      cyan: '#3A8B8C',
    },
  },
  blue: {
    light: {
      primary: '#5A7D9A',
      hover: '#4A6D8A',
      muted: 'rgba(90, 125, 154, 0.12)',
      cyan: '#4A8FA8',
    },
    dark: {
      primary: '#6B9DC0',
      hover: '#85B3D4',
      muted: 'rgba(107, 157, 192, 0.15)',
      cyan: '#5AB0C8',
    },
  },
  purple: {
    light: {
      primary: '#7B6FA5',
      hover: '#6A5F94',
      muted: 'rgba(123, 111, 165, 0.12)',
      cyan: '#8B7FB5',
    },
    dark: {
      primary: '#9B8FC5',
      hover: '#B0A6D8',
      muted: 'rgba(155, 143, 197, 0.15)',
      cyan: '#A89FD0',
    },
  },
  orange: {
    light: {
      primary: '#B8834A',
      hover: '#A6733A',
      muted: 'rgba(184, 131, 74, 0.12)',
      cyan: '#C4935A',
    },
    dark: {
      primary: '#D4A06A',
      hover: '#E4B88A',
      muted: 'rgba(212, 160, 106, 0.15)',
      cyan: '#E0B080',
    },
  },
};

const FONT_SIZE_ROOT: Record<string, string> = {
  small: '14px',
  medium: '15px',
  large: '16px',
};

/** 将 desktop.appearance 应用到 document 根 CSS 变量 */
export function applyDesktopAppearance(
  settings: YunxiSettings | null | undefined,
  resolved: ResolvedTheme,
): void {
  const root = document.documentElement;
  const appearance = getDesktop(settings).appearance;
  const accentId = (appearance?.accentColor ?? 'sage') as AccentId;
  const palette = ACCENTS[accentId] ?? ACCENTS.sage;
  const colors = resolved === 'dark' ? palette.dark : palette.light;

  root.style.setProperty('--accent-primary', colors.primary);
  root.style.setProperty('--accent-primary-hover', colors.hover);
  root.style.setProperty('--accent-primary-muted', colors.muted);
  root.style.setProperty('--accent-cyan', colors.cyan);
  root.style.setProperty('--status-success', colors.primary);
  root.style.setProperty('--ring', colors.primary);
  root.style.setProperty('--primary', colors.primary);

  const density = appearance?.density ?? 'default';
  root.dataset.density = density;

  const fontSize = appearance?.fontSize ?? 'medium';
  root.dataset.fontSize = fontSize;
  root.style.fontSize = FONT_SIZE_ROOT[fontSize] ?? FONT_SIZE_ROOT.medium;

  const animations = appearance?.animations ?? true;
  root.dataset.animations = animations ? 'on' : 'off';

  root.style.setProperty(
    '--editor-font-family',
    editorFontStack(appearance?.editorFont ?? 'jetbrains-mono'),
  );
}

function editorFontStack(id: string): string {
  switch (id) {
    case 'fira-code':
      return '"Fira Code", ui-monospace, monospace';
    case 'sf-mono':
      return '"SF Mono", ui-monospace, monospace';
    case 'system':
      return 'ui-monospace, monospace';
    default:
      return '"JetBrains Mono", ui-monospace, monospace';
  }
}
