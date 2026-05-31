import { useState, useEffect } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import {
  RotateCcw,
  Command,
  Search,
  Sidebar,
  PanelRight,
  FilePlus,
  Sun,
  Terminal,
  Settings,
} from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import { getDesktop } from '@/utils/desktopSettings';
import {
  DEFAULT_SHORTCUT_RECORDS,
  shortcutsFromSettings,
  type ShortcutRecord,
} from '@/utils/shortcutDefaults';

const ICON_BY_ID: Record<string, React.ReactNode> = {
  'new-session': <FilePlus size={14} />,
  'toggle-sidebar': <Sidebar size={14} />,
  'toggle-ai-panel': <PanelRight size={14} />,
  'toggle-terminal': <Terminal size={14} />,
  'toggle-theme': <Sun size={14} />,
  'open-settings': <Settings size={14} />,
  'focus-search': <Search size={14} />,
  'command-palette': <Command size={14} />,
};

interface ShortcutItem extends ShortcutRecord {
  icon: React.ReactNode;
}

function withIcons(records: ShortcutRecord[]): ShortcutItem[] {
  return records.map((r) => ({
    ...r,
    icon: ICON_BY_ID[r.id] ?? <Command size={14} />,
  }));
}

const categories = ['全部', '应用', '导航'];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.02 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 6 },
  show: { opacity: 1, y: 0, transition: { duration: 0.15, ease: 'easeOut' as const } },
};

const ShortcutsSettings: FC = () => {
  const { yunxiSettings, settingsReady, updateDesktopSection } = useApp();
  const [shortcuts, setShortcuts] = useState<ShortcutItem[]>(() =>
    withIcons(DEFAULT_SHORTCUT_RECORDS),
  );
  const [activeCategory, setActiveCategory] = useState('全部');
  const [editingId, setEditingId] = useState<string | null>(null);

  useEffect(() => {
    if (!settingsReady) return;
    const raw = getDesktop(yunxiSettings).shortcuts;
    setShortcuts(withIcons(shortcutsFromSettings(raw)));
  }, [yunxiSettings, settingsReady]);

  const persist = (next: ShortcutItem[]) => {
    const records: ShortcutRecord[] = next.map(({ id, name, keys, category }) => ({
      id,
      name,
      keys,
      category,
    }));
    void updateDesktopSection('shortcuts', records);
  };

  const filteredShortcuts =
    activeCategory === '全部'
      ? shortcuts
      : shortcuts.filter((s) => s.category === activeCategory);

  const handleReset = () => {
    const next = withIcons(DEFAULT_SHORTCUT_RECORDS);
    setShortcuts(next);
    persist(next);
  };

  const handleStartEdit = (id: string) => {
    setEditingId(id);
    setTimeout(() => setEditingId(null), 3000);
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="flex flex-col"
      style={{ padding: '24px 28px' }}
    >
      <motion.div variants={itemVariants} className="mb-5">
        <div className="flex items-center justify-between">
          <div>
            <h2
              style={{
                fontSize: 18,
                fontWeight: 600,
                color: 'var(--text-primary)',
                letterSpacing: '-0.01em',
                lineHeight: 1.4,
                marginBottom: 4,
              }}
            >
              快捷键
            </h2>
            <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
              保存至 .yunxi/settings.json（desktop.shortcuts）。点击条目可高亮（录制待完善）。
            </p>
          </div>
          <button
            type="button"
            onClick={handleReset}
            className="flex items-center gap-1.5 px-3 py-1.5 transition-colors"
            style={{
              borderRadius: 6,
              border: '1px solid var(--border-primary)',
              fontSize: 12,
              color: 'var(--text-secondary)',
            }}
          >
            <RotateCcw size={14} />
            恢复默认
          </button>
        </div>
      </motion.div>

      <motion.div variants={itemVariants} className="mb-4 flex gap-2">
        {categories.map((cat) => (
          <button
            key={cat}
            type="button"
            onClick={() => setActiveCategory(cat)}
            style={{
              padding: '4px 10px',
              fontSize: 11,
              borderRadius: 6,
              backgroundColor:
                activeCategory === cat ? 'var(--accent-primary-muted)' : 'transparent',
              color: activeCategory === cat ? 'var(--accent-primary)' : 'var(--text-tertiary)',
              border:
                activeCategory === cat
                  ? '1px solid var(--accent-primary)'
                  : '1px solid var(--border-primary)',
            }}
          >
            {cat}
          </button>
        ))}
      </motion.div>

      <div className="flex flex-col gap-1">
        {filteredShortcuts.map((shortcut) => (
          <motion.button
            key={shortcut.id}
            type="button"
            variants={itemVariants}
            onClick={() => handleStartEdit(shortcut.id)}
            className="flex w-full items-center justify-between text-left transition-colors"
            style={{
              padding: '10px 12px',
              borderRadius: 8,
              backgroundColor:
                editingId === shortcut.id ? 'var(--accent-primary-muted)' : 'var(--bg-surface)',
              border: '1px solid var(--border-primary)',
            }}
          >
            <div className="flex items-center gap-3">
              <span style={{ color: 'var(--text-tertiary)' }}>{shortcut.icon}</span>
              <span style={{ fontSize: 13, color: 'var(--text-primary)' }}>{shortcut.name}</span>
              <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
                {shortcut.category}
              </span>
            </div>
            <div className="flex items-center gap-1">
              {shortcut.keys.map((key, i) => (
                <kbd
                  key={`${shortcut.id}-${i}`}
                  style={{
                    padding: '2px 6px',
                    fontSize: 11,
                    borderRadius: 4,
                    backgroundColor: 'var(--bg-elevated)',
                    border: '1px solid var(--border-primary)',
                    color: 'var(--text-secondary)',
                    fontFamily: 'ui-monospace, monospace',
                  }}
                >
                  {key}
                </kbd>
              ))}
            </div>
          </motion.button>
        ))}
      </div>
    </motion.div>
  );
};

export default ShortcutsSettings;
