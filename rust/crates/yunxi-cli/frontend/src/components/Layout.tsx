import type { FC, ReactNode } from 'react';
import TitleBar from './TitleBar';
import StatusBar from './StatusBar';

interface LayoutProps {
  children: ReactNode;
  showTitleBar?: boolean;
  showStatusBar?: boolean;
  /** app: VS Code 式（活动栏+侧栏+编辑器+AI）；full: 设置等全宽单页 */
  contentMode?: 'app' | 'full';
}

const Layout: FC<LayoutProps> = ({
  children,
  showTitleBar = true,
  showStatusBar = true,
  contentMode = 'app',
}) => {
  return (
    <div
      className="w-screen overflow-hidden"
      style={{
        height: '100vh',
        display: 'grid',
        gridTemplateColumns: 'auto 1fr auto',
        gridTemplateRows: '38px 1fr 28px',
        gridTemplateAreas: `
          "titlebar titlebar titlebar"
          "left main right"
          "statusbar statusbar statusbar"
        `,
      }}
    >
      {/* Title Bar */}
      {showTitleBar && (
        <div style={{ gridArea: 'titlebar' }}>
          <TitleBar />
        </div>
      )}

      {/* Main Content Area */}
      <main
        className="overflow-hidden"
        style={{
          gridArea: 'main',
          backgroundColor: 'var(--bg-base)',
          minWidth: 0,
          ...(contentMode === 'full'
            ? {
                display: 'flex',
                flexDirection: 'column',
                width: '100%',
              }
            : {
                display: 'grid',
                gridTemplateColumns: 'auto 1fr auto',
              }),
        }}
      >
        {children}
      </main>

      {/* Status Bar */}
      {showStatusBar && (
        <div style={{ gridArea: 'statusbar' }}>
          <StatusBar />
        </div>
      )}
    </div>
  );
};

export default Layout;
