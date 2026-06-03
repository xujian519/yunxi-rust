import { useState, useCallback, useEffect } from 'react';
import type { FC } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useApp } from '@/context/AppProvider';
import SettingsLayout, { type SettingsCategory } from '@/components/settings/SettingsLayout';
import GeneralSettings from '@/components/settings/categories/GeneralSettings';
import ModelSettings from '@/components/settings/categories/ModelSettings';
import AppearanceSettings from '@/components/settings/categories/AppearanceSettings';
import EditorSettings from '@/components/settings/categories/EditorSettings';
import ShortcutsSettings from '@/components/settings/categories/ShortcutsSettings';
import CostSettings from '@/components/settings/categories/CostSettings';
import HooksSettings from '@/components/settings/categories/HooksSettings';
import AboutSettings from '@/components/settings/categories/AboutSettings';

const categoryComponents: Record<SettingsCategory, FC> = {
  general: GeneralSettings,
  model: ModelSettings,
  hooks: HooksSettings,
  appearance: AppearanceSettings,
  editor: EditorSettings,
  shortcuts: ShortcutsSettings,
  cost: CostSettings,
  about: AboutSettings,
};

const Settings: FC = () => {
  const { reloadYunxiSettings } = useApp();
  const [activeCategory, setActiveCategory] = useState<SettingsCategory>('general');

  useEffect(() => {
    void reloadYunxiSettings();
  }, [reloadYunxiSettings]);

  const handleCategoryChange = useCallback((category: SettingsCategory) => {
    setActiveCategory(category);
  }, []);

  const ActiveComponent = categoryComponents[activeCategory];

  return (
    <motion.div
      className="w-full h-full flex-1 min-w-0"
      style={{ flex: 1, width: '100%', minWidth: 0 }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.25, ease: [0.16, 1, 0.3, 1] as [number, number, number, number] }}
    >
      <SettingsLayout
        activeCategory={activeCategory}
        onCategoryChange={handleCategoryChange}
      >
        <AnimatePresence mode="wait">
          <motion.div
            key={activeCategory}
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{
              duration: 0.15,
              ease: [0.16, 1, 0.3, 1] as [number, number, number, number],
            }}
          >
            <ActiveComponent />
          </motion.div>
        </AnimatePresence>
      </SettingsLayout>
    </motion.div>
  );
};

export default Settings;
