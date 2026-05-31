import type { FC } from 'react';
import { useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import Layout from '@/components/Layout';
import ResizablePanel from '@/components/ResizablePanel';
import ActivityBar from '@/components/workbench/ActivityBar';
import ExplorerSidebar from '@/components/workbench/ExplorerSidebar';
import WorkbenchColumn from '@/components/workbench/WorkbenchColumn';
import RightPanel from '@/components/chat/RightPanel';
import PermissionModal from '@/components/chat/PermissionModal';
import DesktopShortcutHandler from '@/components/DesktopShortcutHandler';
import CommandPalette from '@/components/workbench/CommandPalette';
import ImportMaterialsDialog from '@/components/workbench/ImportMaterialsDialog';
import { useApp } from '@/context/AppProvider';
import { loadRightPanelWidth, saveRightPanelWidth } from '@/utils/workspaceStorage';

const RIGHT_PANEL_MIN = 320;
const RIGHT_PANEL_MAX = 560;
const RIGHT_PANEL_DEFAULT = 380;

const MainAppContent: FC = () => {
  const {
    pendingPermission,
    respondPermission,
    commandPaletteOpen,
    setCommandPaletteOpen,
    importMaterialsPreview,
    importMaterialsLoading,
    dismissImportMaterialsPreview,
    confirmImportMaterialsPreview,
  } = useApp();
  const [leftExpanded, setLeftExpanded] = useState(true);
  const [rightVisible, setRightVisible] = useState(true);
  const [rightWidth, setRightWidth] = useState(() =>
    loadRightPanelWidth(RIGHT_PANEL_DEFAULT),
  );

  const handleToggleLeft = useCallback(() => {
    setLeftExpanded((prev) => !prev);
  }, []);

  const handleToggleRight = useCallback(() => {
    setRightVisible((prev) => !prev);
  }, []);

  const handleRightWidthChange = useCallback((width: number) => {
    setRightWidth(width);
  }, []);

  const handleRightWidthCommit = useCallback((width: number) => {
    saveRightPanelWidth(width);
  }, []);

  return (
    <Layout>
      <DesktopShortcutHandler
        onToggleSidebar={handleToggleLeft}
        onToggleAiPanel={handleToggleRight}
      />
      <CommandPalette
        open={commandPaletteOpen}
        onOpenChange={setCommandPaletteOpen}
        onToggleSidebar={handleToggleLeft}
        onToggleAiPanel={handleToggleRight}
      />
      <ImportMaterialsDialog
        preview={importMaterialsPreview}
        loading={importMaterialsLoading}
        onOpenChange={(open) => {
          if (!open) dismissImportMaterialsPreview();
        }}
        onConfirm={() => void confirmImportMaterialsPreview()}
      />
      {pendingPermission ? (
        <PermissionModal
          pending={pendingPermission}
          onRespond={(outcome) => void respondPermission(outcome)}
        />
      ) : null}

      {/* VS Code 式：活动栏 + 资源管理器 */}
      <div className="flex h-full min-w-0 shrink-0 overflow-hidden">
        <ActivityBar onShowExplorer={() => setLeftExpanded(true)} />
        {leftExpanded ? (
          <ResizablePanel
            defaultWidth={260}
            minWidth={200}
            maxWidth={420}
            side="left"
            style={{ height: '100%' }}
          >
            <ExplorerSidebar
              isExpanded
              onToggleExpand={handleToggleLeft}
            />
          </ResizablePanel>
        ) : null}
      </div>

      {/* 中央编辑器工作区 — flex-1 随右侧面板宽度变化自动伸缩 */}
      <WorkbenchColumn />

      {/* 右侧 AI 助手：宽度由 ResizablePanel 单一控制，与中间栏联动 */}
      <AnimatePresence>
        {rightVisible && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="h-full flex-shrink-0 overflow-hidden"
          >
            <ResizablePanel
              width={rightWidth}
              defaultWidth={RIGHT_PANEL_DEFAULT}
              minWidth={RIGHT_PANEL_MIN}
              maxWidth={RIGHT_PANEL_MAX}
              side="right"
              style={{ height: '100%' }}
              onWidthChange={handleRightWidthChange}
              onWidthCommit={handleRightWidthCommit}
            >
              <RightPanel width={rightWidth} onClose={handleToggleRight} />
            </ResizablePanel>
          </motion.div>
        )}
      </AnimatePresence>

      {!rightVisible && (
        <motion.button
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          type="button"
          onClick={handleToggleRight}
          className="absolute right-0 top-1/2 z-20"
          style={{
            width: 20,
            height: 40,
            transform: 'translateY(-50%)',
            backgroundColor: 'var(--bg-elevated)',
            border: '1px solid var(--border-primary)',
            borderRight: 'none',
            borderRadius: '6px 0 0 6px',
            color: 'var(--text-tertiary)',
          }}
          title="打开 AI 助手"
        >
          ◂
        </motion.button>
      )}
    </Layout>
  );
};

const MainApp: FC = () => <MainAppContent />;

export default MainApp;
