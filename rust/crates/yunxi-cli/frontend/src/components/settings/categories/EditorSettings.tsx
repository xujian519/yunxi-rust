import { useState, useEffect } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import ToggleSetting from '../ToggleSetting';
import SelectSetting from '../SelectSetting';
import { useApp } from '@/context/AppProvider';
import { getDesktop, type DesktopEditor } from '@/utils/desktopSettings';

const tabSizes = [
  { value: '2', label: '2 空格' },
  { value: '4', label: '4 空格' },
];

const defaultViews = [
  { value: 'claims', label: '权利要求' },
  { value: 'description', label: '说明书' },
  { value: 'drawings', label: '附图' },
  { value: 'last', label: '上次关闭' },
];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.04 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.2, ease: 'easeOut' as const } },
};

const EditorSettings: FC = () => {
  const { yunxiSettings, settingsReady, updateDesktopSection } = useApp();
  const [tabSize, setTabSize] = useState('4');
  const [wordWrap, setWordWrap] = useState(true);
  const [lineNumbers, setLineNumbers] = useState(true);
  const [highlightActiveLine, setHighlightActiveLine] = useState(true);
  const [autoFormat, setAutoFormat] = useState(false);
  const [autoIndent, setAutoIndent] = useState(true);
  const [showWhitespace, setShowWhitespace] = useState(false);
  const [defaultView, setDefaultView] = useState('last');

  useEffect(() => {
    if (!settingsReady) return;
    const e = getDesktop(yunxiSettings).editor;
    if (e?.tabSize) setTabSize(e.tabSize);
    if (e?.wordWrap != null) setWordWrap(e.wordWrap);
    if (e?.lineNumbers != null) setLineNumbers(e.lineNumbers);
    if (e?.highlightActiveLine != null) setHighlightActiveLine(e.highlightActiveLine);
    if (e?.autoFormat != null) setAutoFormat(e.autoFormat);
    if (e?.autoIndent != null) setAutoIndent(e.autoIndent);
    if (e?.showWhitespace != null) setShowWhitespace(e.showWhitespace);
    if (e?.defaultView) setDefaultView(e.defaultView);
  }, [yunxiSettings, settingsReady]);

  const patchEditor = (patch: Partial<DesktopEditor>) => {
    void updateDesktopSection('editor', patch);
  };

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="flex flex-col"
      style={{ padding: '24px 28px' }}
    >
      {/* Section Header */}
      <motion.div variants={itemVariants} className="mb-5">
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
          编辑器设置
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          配置专利文档编辑器的行为和显示
        </p>
      </motion.div>

      {/* Tab Size Segmented Control */}
      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          Tab 缩进大小
        </span>
        <div
          className="inline-flex"
          style={{
            borderRadius: 8,
            border: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-surface)',
            padding: 3,
            width: 'fit-content',
          }}
        >
          {tabSizes.map((ts) => {
            const isActive = tabSize === ts.value;
            return (
              <button
                key={ts.value}
                onClick={() => {
                  setTabSize(ts.value);
                  patchEditor({ tabSize: ts.value });
                }}
                className="px-5 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  fontSize: 12,
                  fontWeight: isActive ? 500 : 400,
                  color: isActive ? 'var(--text-inverse)' : 'var(--text-secondary)',
                  backgroundColor: isActive ? 'var(--accent-primary)' : 'transparent',
                  border: 'none',
                }}
                type="button"
              >
                {ts.label}
              </button>
            );
          })}
        </div>
      </motion.div>

      {/* Default View */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="默认打开视图"
          value={defaultView}
          options={defaultViews}
          onChange={(v) => {
            setDefaultView(v);
            patchEditor({ defaultView: v });
          }}
        />
      </motion.div>

      {/* Section Separator */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'var(--border-primary)',
          margin: '12px 0',
        }}
      />

      {/* Toggles */}
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="自动换行"
          checked={wordWrap}
          onChange={(v) => {
            setWordWrap(v);
            patchEditor({ wordWrap: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="显示行号"
          checked={lineNumbers}
          onChange={(v) => {
            setLineNumbers(v);
            patchEditor({ lineNumbers: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="高亮当前行"
          checked={highlightActiveLine}
          onChange={(v) => {
            setHighlightActiveLine(v);
            patchEditor({ highlightActiveLine: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="自动缩进"
          description="根据上下文自动调整缩进"
          checked={autoIndent}
          onChange={(v) => {
            setAutoIndent(v);
            patchEditor({ autoIndent: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="显示空白字符"
          description="显示空格和制表符"
          checked={showWhitespace}
          onChange={(v) => {
            setShowWhitespace(v);
            patchEditor({ showWhitespace: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="自动格式化专利文档"
          description="保存时自动调整专利格式"
          checked={autoFormat}
          onChange={(v) => {
            setAutoFormat(v);
            patchEditor({ autoFormat: v });
          }}
        />
      </motion.div>
    </motion.div>
  );
};

export default EditorSettings;
