import type { FC } from 'react';
import { motion } from 'framer-motion';

const ChatView: FC = () => {
  return (
    <motion.div
      initial={{ opacity: 0, x: 16 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.25, ease: [0.4, 0, 0.2, 1] as [number, number, number, number] }}
      className="flex h-full flex-col items-center justify-center"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      <div className="text-center" style={{ maxWidth: 480, padding: 32 }}>
        <motion.img
          src="./app-icon.png"
          alt="云熙"
          className="mx-auto"
          style={{
            width: 80,
            height: 80,
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
            fontSize: 18,
            fontWeight: 600,
            color: 'var(--text-primary)',
            marginBottom: 8,
            letterSpacing: '-0.01em',
          }}
        >
          云熙智能助手
        </h2>
        <p
          style={{
            fontSize: 13,
            color: 'var(--text-secondary)',
            lineHeight: 1.6,
            marginBottom: 20,
          }}
        >
          你好！我是云熙，你的专利智能助手。我可以帮你检索、分析专利，或者辅助撰写专利文档。
        </p>
        <div className="flex flex-wrap justify-center" style={{ gap: 8 }}>
          {['检索相关专利', '分析权利要求', '生成说明书草案', '查看/help命令'].map((chip, idx) => (
            <motion.button
              key={chip}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 + idx * 0.05 }}
              className="transition-all duration-150"
              style={{
                padding: '8px 14px',
                fontSize: 12,
                color: 'var(--text-secondary)',
                backgroundColor: 'var(--bg-elevated)',
                border: '1px solid var(--border-primary)',
                borderRadius: 9999,
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = 'var(--accent-primary)';
                e.currentTarget.style.color = 'var(--accent-primary)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = 'var(--border-primary)';
                e.currentTarget.style.color = 'var(--text-secondary)';
              }}
              type="button"
            >
              {chip}
            </motion.button>
          ))}
        </div>
      </div>
    </motion.div>
  );
};

export default ChatView;
