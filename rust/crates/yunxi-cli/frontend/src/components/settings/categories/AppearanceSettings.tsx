import { useEffect, useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { Sun, Moon, Monitor } from 'lucide-react';
import SelectSetting from '../SelectSetting';
import ToggleSetting from '../ToggleSetting';
import { useTheme, type ThemeMode } from '@/context/ThemeProvider';
import { useApp } from '@/context/AppProvider';
import { getDesktop, type DesktopAppearance } from '@/utils/desktopSettings';

const fontFamilies = [
  { value: 'jetbrains-mono', label: 'JetBrains Mono' },
  { value: 'fira-code', label: 'Fira Code' },
  { value: 'sf-mono', label: 'SF Mono' },
  { value: 'system', label: '系统默认' },
];

const themes = [
  { value: 'light', label: '浅色', icon: Sun },
  { value: 'dark', label: '深色', icon: Moon },
  { value: 'system', label: '跟随系统', icon: Monitor },
];

const densities = [
  { value: 'compact', label: '紧凑' },
  { value: 'default', label: '默认' },
  { value: 'comfortable', label: '宽松' },
];

const fontSizes = [
  { value: 'small', label: '小' },
  { value: 'medium', label: '中' },
  { value: 'large', label: '大' },
];

const accentColors = [
  { value: 'sage', label: ' sage绿', color: '#4A7C6F', darkColor: '#5FA08F' },
  { value: 'blue', label: ' 蓝色', color: '#5A7D9A', darkColor: '#6B9DC0' },
  { value: 'purple', label: ' 紫色', color: '#7B6FA5', darkColor: '#9B8FC5' },
  { value: 'orange', label: ' 橙色', color: '#B8834A', darkColor: '#D4A06A' },
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

const AppearanceSettings: FC = () => {
  const { mode, setMode } = useTheme();
  const { yunxiSettings, settingsReady, updateDesktopSection } = useApp();
  const desk = getDesktop(yunxiSettings).appearance;

  const [fontSize, setFontSize] = useState(desk?.fontSize ?? 'medium');
  const [editorFont, setEditorFont] = useState(desk?.editorFont ?? 'jetbrains-mono');
  const [density, setDensity] = useState(desk?.density ?? 'default');
  const [accentColor, setAccentColor] = useState(desk?.accentColor ?? 'sage');
  const [animations, setAnimations] = useState(desk?.animations ?? true);

  useEffect(() => {
    if (!settingsReady) return;
    const a = getDesktop(yunxiSettings).appearance;
    if (a?.fontSize) setFontSize(a.fontSize);
    if (a?.editorFont) setEditorFont(a.editorFont);
    if (a?.density) setDensity(a.density);
    if (a?.accentColor) setAccentColor(a.accentColor);
    if (a?.animations != null) setAnimations(a.animations);
    if (a?.theme) setMode(a.theme);
  }, [yunxiSettings, settingsReady, setMode]);

  const persistAppearance = (patch: Partial<DesktopAppearance>) => {
    void updateDesktopSection('appearance', patch);
  };

  const handleTheme = (value: ThemeMode) => {
    setMode(value);
    persistAppearance({ theme: value });
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
          外观设置
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          自定义应用的主题、字体和界面密度（保存至 .yunxi/settings.json）
        </p>
      </motion.div>

      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          主题
        </span>
        <div className="grid grid-cols-3 gap-2">
          {themes.map(({ value, label, icon: Icon }) => {
            const active = mode === value;
            return (
              <button
                key={value}
                type="button"
                onClick={() => handleTheme(value as ThemeMode)}
                className="flex flex-col items-center gap-2 transition-colors"
                style={{
                  padding: '12px 8px',
                  borderRadius: 8,
                  border: active
                    ? '2px solid var(--accent-primary)'
                    : '1px solid var(--border-primary)',
                  backgroundColor: active ? 'var(--accent-primary-muted)' : 'var(--bg-surface)',
                }}
              >
                <Icon size={20} style={{ color: 'var(--text-secondary)' }} />
                <span style={{ fontSize: 11, color: 'var(--text-secondary)' }}>{label}</span>
              </button>
            );
          })}
        </div>
      </motion.div>

      <motion.div variants={itemVariants}>
        <SelectSetting
          label="界面字号"
          description="调整全局 UI 字号"
          value={fontSize}
          options={fontSizes}
          onChange={(v) => {
            setFontSize(v);
            persistAppearance({ fontSize: v });
          }}
        />
      </motion.div>

      <motion.div variants={itemVariants}>
        <SelectSetting
          label="编辑器字体"
          description="代码与 Markdown 编辑区字体"
          value={editorFont}
          options={fontFamilies}
          onChange={(v) => {
            setEditorFont(v);
            persistAppearance({ editorFont: v });
          }}
        />
      </motion.div>

      <motion.div variants={itemVariants}>
        <SelectSetting
          label="界面密度"
          description="控制列表与面板的间距"
          value={density}
          options={densities}
          onChange={(v) => {
            setDensity(v);
            persistAppearance({ density: v });
          }}
        />
      </motion.div>

      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <span style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-primary)' }}>
          强调色
        </span>
        <div className="flex gap-2">
          {accentColors.map(({ value, label, color, darkColor }) => {
            const active = accentColor === value;
            return (
              <button
                key={value}
                type="button"
                title={label.trim()}
                onClick={() => {
                  setAccentColor(value);
                  persistAppearance({ accentColor: value });
                }}
                style={{
                  width: 32,
                  height: 32,
                  borderRadius: '50%',
                  backgroundColor: color,
                  border: active ? '2px solid var(--text-primary)' : '2px solid transparent',
                  boxShadow: active ? `0 0 0 2px ${darkColor}` : 'none',
                }}
              />
            );
          })}
        </div>
      </motion.div>

      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="界面动画"
          description="启用过渡与微动效"
          checked={animations}
          onChange={(v) => {
            setAnimations(v);
            persistAppearance({ animations: v });
          }}
        />
      </motion.div>
    </motion.div>
  );
};

export default AppearanceSettings;
