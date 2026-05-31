import type { FC, CSSProperties } from 'react';
import { isTauriRuntime } from '@/api';

interface ExtendedCSSProperties extends CSSProperties {
  WebkitAppRegion?: string;
}

interface TitleBarProps {
  /** 桌面端使用系统标题栏，不显示自定义装饰 */
  showTrafficLights?: boolean;
}

/** macOS 原生交通灯占位宽度 */
const MACOS_TRAFFIC_LIGHT_INSET = 78;

const TitleBar: FC<TitleBarProps> = ({ showTrafficLights = true }) => {
  const isDesktop = isTauriRuntime();
  const useNativeChrome = isDesktop;

  const headerStyle: ExtendedCSSProperties = {
    height: 38,
    backgroundColor: 'var(--bg-surface)',
    borderBottom: '1px solid var(--border-primary)',
    userSelect: 'none',
    WebkitAppRegion: 'drag',
  };

  const noDragStyle: ExtendedCSSProperties = { WebkitAppRegion: 'no-drag' };
  const dragStyle: ExtendedCSSProperties = { WebkitAppRegion: 'drag' };

  return (
    <header className="flex items-center select-none shrink-0" style={headerStyle}>
      {/* 左侧：Web 模式显示装饰；桌面端仅留原生按钮区域 */}
      <div className="flex items-center" style={{ paddingLeft: useNativeChrome ? MACOS_TRAFFIC_LIGHT_INSET : 12 }}>
        {!useNativeChrome && showTrafficLights && (
          <div className="flex items-center" style={{ gap: 8, width: 52, ...noDragStyle }}>
            <span style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#FF5F57' }} />
            <span style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#FFBD2E' }} />
            <span style={{ width: 12, height: 12, borderRadius: '50%', backgroundColor: '#28C840' }} />
          </div>
        )}
      </div>

      {/* 中间拖拽区 — 桌面端不再重复应用名（系统标题栏已显示） */}
      <div className="flex flex-1 items-center justify-center" style={dragStyle}>
        {!useNativeChrome && (
          <div className="flex items-center" style={{ gap: 8 }}>
            <img
              src="./app-icon.png"
              alt=""
              style={{ width: 18, height: 18, borderRadius: 4, objectFit: 'cover' }}
            />
            <span style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-secondary)' }}>
              云熙智能体
            </span>
          </div>
        )}
      </div>

      <div style={{ width: useNativeChrome ? MACOS_TRAFFIC_LIGHT_INSET : 12, ...noDragStyle }} />
    </header>
  );
};

export default TitleBar;
