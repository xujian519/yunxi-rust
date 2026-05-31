import type { FC } from 'react';
import EditorWorkbench from '@/components/workbench/EditorWorkbench';
import BottomPanel from '@/components/workbench/BottomPanel';
import ResizablePanelVertical from '@/components/ResizablePanelVertical';
import { useApp } from '@/context/AppProvider';
import { savePanelHeight } from '@/utils/workspaceStorage';

const WorkbenchColumn: FC = () => {
  const { bottomPanelVisible, bottomPanelHeight, setBottomPanelHeight } = useApp();

  return (
    <div className="flex h-full min-w-0 flex-1 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-hidden">
        <EditorWorkbench />
      </div>
      {bottomPanelVisible && (
        <ResizablePanelVertical
          height={bottomPanelHeight}
          minHeight={100}
          maxHeight={420}
          onHeightChange={(h) => {
            setBottomPanelHeight(h);
            savePanelHeight(h);
          }}
        >
          <BottomPanel />
        </ResizablePanelVertical>
      )}
    </div>
  );
};

export default WorkbenchColumn;
