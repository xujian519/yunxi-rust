import type { FC } from 'react';
import { motion } from 'framer-motion';

const WelcomeEditor: FC = () => (
  <div
    className="flex h-full flex-col items-center justify-center"
    style={{ backgroundColor: 'var(--bg-surface)', padding: 32 }}
  >
    <motion.img
      src="./app-icon.png"
      alt="云熙"
      style={{
        width: 72,
        height: 72,
        borderRadius: '50%',
        objectFit: 'cover',
        marginBottom: 20,
        boxShadow: '0 4px 16px rgba(0,0,0,0.08)',
      }}
      animate={{ scale: [1, 1.02, 1] }}
      transition={{ duration: 3, repeat: Infinity, ease: 'easeInOut' }}
    />
    <h2
      style={{
        fontSize: 17,
        fontWeight: 600,
        color: 'var(--text-primary)',
        marginBottom: 8,
      }}
    >
      工作区
    </h2>
    <p
      style={{
        fontSize: 13,
        color: 'var(--text-secondary)',
        textAlign: 'center',
        maxWidth: 420,
        lineHeight: 1.6,
      }}
    >
      在左侧资源管理器中打开案件文档，或使用活动栏打开对比、审查、检索视图。AI 助手始终在右侧面板。
    </p>
  </div>
);

export default WelcomeEditor;
