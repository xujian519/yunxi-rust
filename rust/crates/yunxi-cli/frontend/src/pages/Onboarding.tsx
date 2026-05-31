import { useState, useCallback } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { useNavigate } from 'react-router';
import { Key, Eye, EyeOff, Loader2, ArrowRight } from 'lucide-react';
import MeshGradient from '@/components/MeshGradient';
import { api, isTauriRuntime } from '@/api';
import { useApp } from '@/context/AppProvider';
import SelectSetting from '@/components/settings/SelectSetting';

const modelOptions = [
  { value: 'deepseek-v4-pro', label: 'DeepSeek-V4 Pro (推荐)' },
  { value: 'deepseek-v4-flash', label: 'DeepSeek-V4 Flash' },
  { value: 'auto', label: '自动选择' },
];

const Onboarding: FC = () => {
  const navigate = useNavigate();
  const { model, reloadYunxiSettings } = useApp();
  const [selectedModel, setSelectedModel] = useState(model || 'deepseek-v4-pro');
  const [apiKey, setApiKey] = useState('');
  const [showKey, setShowKey] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const finish = useCallback(
    async (skipKey: boolean) => {
      setLoading(true);
      setError('');
      try {
        if (!skipKey && isTauriRuntime()) {
          if (!apiKey.trim()) {
            setError('请输入 API Key');
            setLoading(false);
            return;
          }
          await api.saveLlmApiKey(selectedModel, apiKey.trim());
          await reloadYunxiSettings();
        }
        navigate('/', { replace: true });
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    [apiKey, selectedModel, navigate, reloadYunxiSettings],
  );

  return (
    <div
      className="relative flex min-h-screen items-center justify-center overflow-hidden"
      style={{ backgroundColor: 'var(--bg-base)' }}
    >
      <MeshGradient />
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        className="relative z-10 w-full max-w-md px-6"
      >
        <div
          className="rounded-2xl p-8"
          style={{
            backgroundColor: 'var(--glass-bg)',
            backdropFilter: 'var(--glass-backdrop)',
            border: 'var(--glass-border)',
            boxShadow: 'var(--glass-shadow)',
          }}
        >
          <h1
            style={{
              fontSize: 22,
              fontWeight: 600,
              color: 'var(--text-primary)',
              marginBottom: 8,
            }}
          >
            欢迎使用云熙
          </h1>
          <p style={{ fontSize: 13, color: 'var(--text-secondary)', marginBottom: 24, lineHeight: 1.5 }}>
            配置默认模型与 API Key。密钥将写入工作区{' '}
            <code style={{ fontSize: 12 }}>.yunxi/settings.json</code> 的{' '}
            <code style={{ fontSize: 12 }}>env</code> 段，供对话与工具调用使用。
          </p>

          <div className="mb-4">
            <SelectSetting
              label="默认模型"
              value={selectedModel}
              options={modelOptions}
              onChange={setSelectedModel}
            />
          </div>

          <div className="mb-4">
            <label
              style={{
                display: 'block',
                fontSize: 13,
                fontWeight: 500,
                color: 'var(--text-primary)',
                marginBottom: 8,
              }}
            >
              API Key
            </label>
            <div className="relative">
              <Key
                size={16}
                className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2"
                style={{ color: 'var(--text-tertiary)' }}
              />
              <input
                type={showKey ? 'text' : 'password'}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-..."
                disabled={loading}
                className="w-full py-2.5 pl-10 pr-10"
                style={{
                  borderRadius: 8,
                  border: '1px solid var(--border-primary)',
                  backgroundColor: 'var(--bg-surface)',
                  fontSize: 13,
                  color: 'var(--text-primary)',
                  fontFamily: 'ui-monospace, monospace',
                }}
              />
              <button
                type="button"
                onClick={() => setShowKey((v) => !v)}
                className="absolute right-2 top-1/2 -translate-y-1/2 p-1"
                style={{ color: 'var(--text-tertiary)' }}
              >
                {showKey ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
          </div>

          {error ? (
            <p style={{ fontSize: 12, color: 'var(--status-error)', marginBottom: 12 }}>{error}</p>
          ) : null}

          <button
            type="button"
            disabled={loading}
            onClick={() => void finish(false)}
            className="flex w-full items-center justify-center gap-2 py-2.5 transition-opacity"
            style={{
              borderRadius: 8,
              backgroundColor: 'var(--accent-primary)',
              color: 'var(--text-inverse)',
              fontSize: 14,
              fontWeight: 500,
              opacity: loading ? 0.7 : 1,
            }}
          >
            {loading ? <Loader2 size={16} className="animate-spin" /> : <ArrowRight size={16} />}
            开始使用
          </button>

          <button
            type="button"
            disabled={loading}
            onClick={() => void finish(true)}
            className="mt-3 w-full py-2 text-center"
            style={{ fontSize: 12, color: 'var(--text-tertiary)' }}
          >
            稍后配置（预览 / 只读本地）
          </button>
        </div>
      </motion.div>
    </div>
  );
};

export default Onboarding;
