import type { FC, ReactNode } from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';

interface ResizablePanelVerticalProps {
  children: ReactNode;
  height: number;
  minHeight: number;
  maxHeight: number;
  onHeightChange: (height: number) => void;
  className?: string;
}

/** 自底向上拖拽调整高度的面板（VS Code 底部 Panel） */
const ResizablePanelVertical: FC<ResizablePanelVerticalProps> = ({
  children,
  height,
  minHeight,
  maxHeight,
  onHeightChange,
  className = '',
}) => {
  const [isResizing, setIsResizing] = useState(false);
  const startYRef = useRef(0);
  const startHeightRef = useRef(0);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      setIsResizing(true);
      startYRef.current = e.clientY;
      startHeightRef.current = height;
      document.body.style.cursor = 'row-resize';
      document.body.style.userSelect = 'none';
    },
    [height],
  );

  useEffect(() => {
    if (!isResizing) return;

    const onMove = (e: MouseEvent) => {
      const delta = startYRef.current - e.clientY;
      const next = Math.min(maxHeight, Math.max(minHeight, startHeightRef.current + delta));
      onHeightChange(next);
    };

    const onUp = () => {
      setIsResizing(false);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
    return () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    };
  }, [isResizing, minHeight, maxHeight, onHeightChange]);

  return (
    <div
      className={`relative flex flex-shrink-0 flex-col ${className}`}
      style={{
        height,
      }}
    >
      <div
        className="flex cursor-row-resize items-center justify-center"
        style={{
          height: 6,
          flexShrink: 0,
          borderTop: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
        onMouseDown={handleMouseDown}
        role="separator"
        aria-label="调整面板高度"
      >
        <div
          style={{
            width: 32,
            height: 3,
            borderRadius: 2,
            backgroundColor: isResizing ? 'var(--accent-primary)' : 'var(--border-secondary)',
          }}
        />
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
    </div>
  );
};

export default ResizablePanelVertical;
