import { useCallback, useEffect, useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { Terminal } from 'lucide-react';
import { useApp } from '@/context/AppProvider';
import type { YunxiSettings } from '@/api';

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.2, ease: 'easeOut' as const } },
};

function linesToHooks(pre: string, post: string): YunxiSettings['hooks'] {
  const toList = (text: string) =>
    text
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean);
  const preList = toList(pre);
  const postList = toList(post);
  const hooks: NonNullable<YunxiSettings['hooks']> = {};
  if (preList.length) hooks.PreToolUse = preList;
  if (postList.length) hooks.PostToolUse = postList;
  return Object.keys(hooks).length ? hooks : undefined;
}

const HooksSettings: FC = () => {
  const { yunxiSettings, settingsReady, persistYunxiSettings } = useApp();
  const [preToolUse, setPreToolUse] = useState('');
  const [postToolUse, setPostToolUse] = useState('');
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (!settingsReady) return;
    const hooks = yunxiSettings?.hooks;
    setPreToolUse((hooks?.PreToolUse ?? []).join('\n'));
    setPostToolUse((hooks?.PostToolUse ?? []).join('\n'));
  }, [yunxiSettings, settingsReady]);

  const saveHooks = useCallback(async () => {
    if (!yunxiSettings) return;
    const hooks = linesToHooks(preToolUse, postToolUse);
    await persistYunxiSettings({ ...yunxiSettings, hooks });
    setSaved(true);
    window.setTimeout(() => setSaved(false), 2000);
  }, [yunxiSettings, preToolUse, postToolUse, persistYunxiSettings]);

  return (
    <motion.div variants={{ hidden: { opacity: 0 }, show: { opacity: 1, transition: { staggerChildren: 0.05 } } }} initial="hidden" animate="show">
      <motion.div variants={itemVariants} className="mb-6">
        <div className="mb-2 flex items-center" style={{ gap: 8 }}>
          <Terminal size={18} style={{ color: 'var(--accent-primary)' }} />
          <h2 style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-primary)' }}>Hooks 配置</h2>
        </div>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
          与 Claude Code 兼容：在工具调用前/后执行 shell 命令。写入项目 <code>.yunxi/settings.json</code> 的
          <code> hooks</code> 段，Agent 下一轮对话生效。
        </p>
      </motion.div>

      <motion.div variants={itemVariants} className="mb-5">
        <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-primary)' }}>PreToolUse（每行一条命令）</label>
        <textarea
          value={preToolUse}
          onChange={(e) => setPreToolUse(e.target.value)}
          placeholder="例如：/usr/local/bin/lint-check.sh"
          className="mt-2 w-full resize-y rounded-lg border bg-transparent focus:outline-none"
          style={{
            minHeight: 100,
            padding: 12,
            fontSize: 12,
            fontFamily: 'var(--editor-font-family)',
            borderColor: 'var(--border-primary)',
            color: 'var(--text-primary)',
          }}
        />
      </motion.div>

      <motion.div variants={itemVariants} className="mb-5">
        <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-primary)' }}>PostToolUse（每行一条命令）</label>
        <textarea
          value={postToolUse}
          onChange={(e) => setPostToolUse(e.target.value)}
          placeholder="例如：echo tool finished"
          className="mt-2 w-full resize-y rounded-lg border bg-transparent focus:outline-none"
          style={{
            minHeight: 100,
            padding: 12,
            fontSize: 12,
            fontFamily: 'var(--editor-font-family)',
            borderColor: 'var(--border-primary)',
            color: 'var(--text-primary)',
          }}
        />
      </motion.div>

      <motion.div variants={itemVariants} className="flex items-center" style={{ gap: 12 }}>
        <button
          type="button"
          onClick={() => void saveHooks()}
          style={{
            padding: '8px 16px',
            fontSize: 13,
            fontWeight: 600,
            borderRadius: 8,
            border: 'none',
            backgroundColor: 'var(--accent-primary)',
            color: '#fff',
          }}
        >
          保存 Hooks
        </button>
        {saved ? (
          <span style={{ fontSize: 12, color: 'var(--status-success)' }}>已保存</span>
        ) : null}
      </motion.div>
    </motion.div>
  );
};

export default HooksSettings;
