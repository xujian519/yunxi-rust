import type { FC, ReactNode } from 'react';
import { useCallback, useRef, useState, useEffect } from 'react';

interface ResizablePanelProps {
  children: ReactNode;
  /** 受控宽度；传入时由父组件统一管理布局 */
  width?: number;
  defaultWidth: number;
  minWidth: number;
  maxWidth: number;
  side: 'left' | 'right';
  className?: string;
  style?: React.CSSProperties;
  /** 宽度变化回调（实时，拖拽中持续触发） */
  onWidthChange?: (width: number) => void;
  /** 拖拽结束回调（适合持久化） */
  onWidthCommit?: (width: number) => void;
}

const ResizablePanel: FC<ResizablePanelProps> = ({
  children,
  width: controlledWidth,
  defaultWidth,
  minWidth,
  maxWidth,
  side,
  className = '',
  style = {},
  onWidthChange,
  onWidthCommit,
}) => {
  const [internalWidth, setInternalWidth] = useState(defaultWidth);
  const [isResizing, setIsResizing] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const startXRef = useRef(0);
  const startWidthRef = useRef(0);
  const latestWidthRef = useRef(controlledWidth ?? internalWidth);

  const width = controlledWidth ?? internalWidth;

  useEffect(() => {
    latestWidthRef.current = width;
  }, [width]);

  const applyWidth = useCallback(
    (next: number) => {
      const clamped = Math.min(maxWidth, Math.max(minWidth, next));
      if (controlledWidth === undefined) {
        setInternalWidth(clamped);
      }
      latestWidthRef.current = clamped;
      onWidthChange?.(clamped);
    },
    [controlledWidth, maxWidth, minWidth, onWidthChange],
  );

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsResizing(true);
      startXRef.current = e.clientX;
      startWidthRef.current = width;

      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    },
    [width],
  );

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = e.clientX - startXRef.current;
      const newWidth =
        side === 'left'
          ? startWidthRef.current + delta
          : startWidthRef.current - delta;
      applyWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      onWidthCommit?.(latestWidthRef.current);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, applyWidth, onWidthCommit, side]);

  return (
    <div
      ref={panelRef}
      className={`relative h-full flex-shrink-0 ${className}`}
      style={{
        width,
        transition: isResizing ? 'none' : undefined,
        ...style,
      }}
    >
      {children}
      <div
        className="group absolute top-0 bottom-0 z-10 flex items-center justify-center"
        style={{
          [side === 'left' ? 'right' : 'left']: -5,
          width: 12,
          cursor: 'col-resize',
        }}
        onMouseDown={handleMouseDown}
        role="separator"
        aria-label="调整面板宽度"
      >
        <div
          className="rounded transition-all duration-150 group-hover:opacity-70"
          style={{
            width: 3,
            height: isResizing ? '60%' : '20%',
            opacity: isResizing ? 1 : 0.3,
            backgroundColor: isResizing
              ? 'var(--accent-primary)'
              : 'var(--border-secondary)',
          }}
        />
      </div>
      {isResizing && (
        <div
          className="pointer-events-none fixed inset-0 z-50"
          style={{ cursor: 'col-resize' }}
        />
      )}
    </div>
  );
};

export default ResizablePanel;
