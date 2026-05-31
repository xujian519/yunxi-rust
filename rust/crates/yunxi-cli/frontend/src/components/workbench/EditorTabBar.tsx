import type { FC, DragEvent } from 'react';
import { useState } from 'react';
import { X } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import ViewModeToggle from './ViewModeToggle';

const EditorTabBar: FC = () => {
  const { editorTabs, activeTabId, setActiveTab, closeEditorTab, reorderEditorTabs } = useApp();
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dropIndex, setDropIndex] = useState<number | null>(null);

  if (editorTabs.length === 0) return null;

  const onDragStart = (index: number) => (e: DragEvent) => {
    setDragIndex(index);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', String(index));
  };

  const onDragOver = (index: number) => (e: DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDropIndex(index);
  };

  const onDrop = (index: number) => (e: DragEvent) => {
    e.preventDefault();
    const from = dragIndex ?? Number(e.dataTransfer.getData('text/plain'));
    if (!Number.isNaN(from)) reorderEditorTabs(from, index);
    setDragIndex(null);
    setDropIndex(null);
  };

  const onDragEnd = () => {
    setDragIndex(null);
    setDropIndex(null);
  };

  return (
    <div
      className="flex items-stretch overflow-x-auto custom-scrollbar"
      style={{
        height: 35,
        minHeight: 35,
        backgroundColor: 'var(--bg-elevated)',
        borderBottom: '1px solid var(--border-primary)',
      }}
    >
      {editorTabs.map((tab, index) => {
        const isActive = tab.id === activeTabId;
        const isDropTarget = dropIndex === index && dragIndex !== null && dragIndex !== index;
        return (
          <div
            key={tab.id}
            draggable
            onDragStart={onDragStart(index)}
            onDragOver={onDragOver(index)}
            onDrop={onDrop(index)}
            onDragEnd={onDragEnd}
            className="group flex max-w-[200px] min-w-0 flex-shrink-0 items-center"
            style={{
              borderRight: '1px solid var(--border-primary)',
              backgroundColor: isActive
                ? 'var(--bg-surface)'
                : isDropTarget
                  ? 'var(--accent-primary-muted)'
                  : 'transparent',
              opacity: dragIndex === index ? 0.5 : 1,
              cursor: 'grab',
            }}
          >
            <button
              type="button"
              onClick={() => setActiveTab(tab.id)}
              className="flex min-w-0 flex-1 items-center truncate transition-colors"
              style={{
                height: 35,
                padding: '0 10px',
                fontSize: 12,
                color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)',
                fontWeight: isActive ? 500 : 400,
              }}
              title={tab.title}
            >
              <span className="truncate">{tab.title}</span>
            </button>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                closeEditorTab(tab.id);
              }}
              className="flex items-center justify-center opacity-0 transition-opacity group-hover:opacity-100"
              style={{
                width: 28,
                height: 35,
                color: 'var(--text-tertiary)',
                flexShrink: 0,
              }}
              aria-label={`关闭 ${tab.title}`}
            >
              <X size={14} />
            </button>
          </div>
        );
      })}
      {editorTabs.length > 0 && (
        <div className="flex flex-1 items-center justify-end px-2">
          <ViewModeToggle />
        </div>
      )}
    </div>
  );
};

export default EditorTabBar;
