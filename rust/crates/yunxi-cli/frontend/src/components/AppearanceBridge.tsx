import { useEffect } from 'react';
import type { FC } from 'react';
import { useApp } from '@/context/AppProvider';
import { useTheme } from '@/context/ThemeProvider';
import { applyDesktopAppearance } from '@/utils/applyDesktopAppearance';

/** 将 settings.desktop.appearance 同步到 CSS 变量 */
const AppearanceBridge: FC = () => {
  const { yunxiSettings } = useApp();
  const { resolved } = useTheme();

  useEffect(() => {
    applyDesktopAppearance(yunxiSettings, resolved);
  }, [yunxiSettings, resolved]);

  return null;
};

export default AppearanceBridge;
