import { useState, useEffect, useCallback } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import { Download, Upload, Trash2, AlertTriangle, Server } from 'lucide-react';
import SelectSetting from '../SelectSetting';
import ToggleSetting from '../ToggleSetting';
import { useApp } from '@/context/AppProvider';
import { api, hasBackendTools } from '@/api';
import type { McpStatusReport } from '@/api';
import {
  getDesktop,
  readPermissionMode,
  withPermissionMode,
  type PermissionDefaultMode,
  type DesktopGeneral,
} from '@/utils/desktopSettings';

const languages = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en', label: 'English' },
  { value: 'ja', label: '日本語' },
];

const patentOffices = [
  { value: 'CNIPA', label: '中国专利 (CNIPA)' },
  { value: 'USPTO', label: '美国专利 (USPTO)' },
  { value: 'EPO', label: '欧洲专利 (EPO)' },
  { value: 'WIPO', label: 'WIPO' },
];

const sessionDurations = [
  { value: '7', label: '7天' },
  { value: '30', label: '30天' },
  { value: '90', label: '90天' },
  { value: 'forever', label: '永久' },
];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.04,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 8 },
  show: { opacity: 1, y: 0, transition: { duration: 0.2, ease: 'easeOut' as const } },
};

const permissionModes = [
  { value: 'dontAsk', label: '自动批准（dontAsk）' },
  { value: 'plan', label: '计划模式（plan）' },
  { value: 'read-only', label: '只读（read-only）' },
  { value: 'workspace-write', label: '工作区写入（workspace-write）' },
];

const GeneralSettings: FC = () => {
  const { yunxiSettings, settingsReady, updateDesktopSection, persistYunxiSettings } = useApp();
  const [language, setLanguage] = useState('zh-CN');
  const [patentOffice, setPatentOffice] = useState('CNIPA');
  const [sessionDuration, setSessionDuration] = useState('30');
  const [autoSave, setAutoSave] = useState(true);
  const [notifications, setNotifications] = useState(true);
  const [soundEffects, setSoundEffects] = useState(false);
  const [permissionMode, setPermissionMode] = useState<PermissionDefaultMode>('dontAsk');
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [mcpStatus, setMcpStatus] = useState<McpStatusReport | null>(null);
  const [mcpLoading, setMcpLoading] = useState(false);

  const refreshMcp = useCallback(async () => {
    if (!hasBackendTools()) return;
    setMcpLoading(true);
    try {
      setMcpStatus(await api.getMcpStatus());
    } catch {
      setMcpStatus(null);
    } finally {
      setMcpLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshMcp();
  }, [refreshMcp]);

  useEffect(() => {
    if (!settingsReady) return;
    const g = getDesktop(yunxiSettings).general;
    if (g?.language) setLanguage(g.language);
    if (g?.patentOffice) setPatentOffice(g.patentOffice);
    if (g?.sessionDuration) setSessionDuration(g.sessionDuration);
    if (g?.autoSave != null) setAutoSave(g.autoSave);
    if (g?.notifications != null) setNotifications(g.notifications);
    if (g?.soundEffects != null) setSoundEffects(g.soundEffects);
    setPermissionMode(readPermissionMode(yunxiSettings));
  }, [yunxiSettings, settingsReady]);

  const patchGeneral = (patch: Partial<DesktopGeneral>) => {
    void updateDesktopSection('general', patch);
  };

  const handleExport = () => {
    const data = { export: true, timestamp: new Date().toISOString() };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `yunxi-backup-${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
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
          通用设置
        </h2>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          配置应用的基本行为和偏好
        </p>
      </motion.div>

      {/* Language */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="界面语言"
          value={language}
          options={languages}
          onChange={(v) => {
            setLanguage(v);
            patchGeneral({ language: v });
          }}
        />
      </motion.div>

      {/* Patent Office */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="默认专利数据库"
          value={patentOffice}
          options={patentOffices}
          onChange={(v) => {
            setPatentOffice(v);
            patchGeneral({ patentOffice: v });
          }}
        />
      </motion.div>

      {/* Session Duration */}
      <motion.div variants={itemVariants}>
        <SelectSetting
          label="会话保留时长"
          value={sessionDuration}
          options={sessionDurations}
          onChange={(v) => {
            setSessionDuration(v);
            patchGeneral({ sessionDuration: v });
          }}
        />
      </motion.div>

      <motion.div variants={itemVariants}>
        <SelectSetting
          label="工具权限默认模式"
          description="写入 .yunxi/settings.json 的 permissions.defaultMode"
          value={permissionMode}
          options={permissionModes}
          onChange={(v) => {
            const mode = v as PermissionDefaultMode;
            setPermissionMode(mode);
            if (yunxiSettings) {
              void persistYunxiSettings(withPermissionMode(yunxiSettings, mode));
            }
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
          label="自动保存"
          checked={autoSave}
          onChange={(v) => {
            setAutoSave(v);
            patchGeneral({ autoSave: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="通知提醒"
          description="启用桌面通知以获取重要更新"
          checked={notifications}
          onChange={(v) => {
            setNotifications(v);
            patchGeneral({ notifications: v });
          }}
        />
      </motion.div>
      <motion.div variants={itemVariants}>
        <ToggleSetting
          label="音效"
          description="启用操作音效反馈"
          checked={soundEffects}
          onChange={(v) => {
            setSoundEffects(v);
            patchGeneral({ soundEffects: v });
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

      {/* Data Management */}
      <motion.div variants={itemVariants} className="flex flex-col gap-3 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text-primary)',
            lineHeight: 1.4,
          }}
        >
          数据管理
        </span>
        <div className="flex gap-3">
          <button
            onClick={handleExport}
            className="flex items-center gap-2 px-4 py-2 transition-colors"
            style={{
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-primary)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
            }}
            type="button"
          >
            <Download size={14} />
            导出所有数据
          </button>
          <button
            className="flex items-center gap-2 px-4 py-2 transition-colors"
            style={{
              borderRadius: 8,
              border: '1px solid var(--border-primary)',
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-primary)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-surface)';
            }}
            type="button"
          >
            <Upload size={14} />
            导入数据
          </button>
        </div>
      </motion.div>

      {/* MCP */}
      <motion.div variants={itemVariants} className="flex flex-col gap-2 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Server size={14} style={{ color: 'var(--accent-primary)' }} />
            <span style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-primary)' }}>
              MCP 服务器
            </span>
          </div>
          <button
            type="button"
            onClick={() => void refreshMcp()}
            disabled={mcpLoading}
            style={{ fontSize: 11, color: 'var(--accent-primary)' }}
          >
            {mcpLoading ? '刷新中…' : '刷新'}
          </button>
        </div>
        <p style={{ fontSize: 11, color: 'var(--text-tertiary)', lineHeight: 1.5 }}>
          在 <code>.yunxi/settings.json</code> 或项目配置中设置 <code>mcpServers</code>；对话时自动发现并合并 MCP 工具。
        </p>
        {mcpStatus && mcpStatus.servers.length > 0 ? (
          <div className="flex flex-col gap-1">
            {mcpStatus.servers.map((s: McpStatusReport['servers'][number]) => (
              <div
                key={s.name}
                style={{
                  fontSize: 11,
                  padding: '6px 10px',
                  borderRadius: 6,
                  backgroundColor: 'var(--bg-sidebar-active)',
                  color: 'var(--text-secondary)',
                }}
              >
                <strong>{s.name}</strong> — {s.status} · {s.tool_count} 工具 · {s.transport}
              </div>
            ))}
          </div>
        ) : (
          <p style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>未配置 MCP 服务器</p>
        )}
      </motion.div>

      {/* Danger Zone */}
      <motion.div
        variants={itemVariants}
        style={{
          height: 1,
          backgroundColor: 'rgba(184, 92, 80, 0.3)',
          margin: '12px 0',
        }}
      />
      <motion.div variants={itemVariants} className="flex flex-col gap-3 py-3">
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--status-error)',
            lineHeight: 1.4,
          }}
        >
          危险区域
        </span>
        {!showClearConfirm ? (
          <button
            onClick={() => setShowClearConfirm(true)}
            className="flex items-center gap-2 px-4 py-2 transition-colors self-start"
            style={{
              borderRadius: 8,
              border: '1px solid rgba(184, 92, 80, 0.4)',
              backgroundColor: 'transparent',
              color: 'var(--status-error)',
              fontSize: 12,
              fontWeight: 500,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'rgba(184, 92, 80, 0.08)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
            type="button"
          >
            <Trash2 size={14} />
            清除所有数据
          </button>
        ) : (
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="flex flex-col gap-3 p-4"
            style={{
              borderRadius: 10,
              backgroundColor: 'rgba(184, 92, 80, 0.06)',
              border: '1px solid rgba(184, 92, 80, 0.2)',
            }}
          >
            <div className="flex items-start gap-2">
              <AlertTriangle size={16} style={{ color: 'var(--status-error)', marginTop: 2 }} />
              <div>
                <p style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-primary)' }}>
                  确定要清除所有数据吗？
                </p>
                <p style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 2 }}>
                  此操作不可撤销，所有会话、设置和数据将被永久删除。
                </p>
              </div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  border: '1px solid var(--border-primary)',
                  backgroundColor: 'var(--bg-surface)',
                  color: 'var(--text-primary)',
                  fontSize: 12,
                }}
                type="button"
              >
                取消
              </button>
              <button
                onClick={() => setShowClearConfirm(false)}
                className="px-4 py-1.5 transition-colors"
                style={{
                  borderRadius: 6,
                  border: 'none',
                  backgroundColor: 'var(--status-error)',
                  color: '#FFFFFF',
                  fontSize: 12,
                }}
                type="button"
              >
                确认清除
              </button>
            </div>
          </motion.div>
        )}
      </motion.div>
    </motion.div>
  );
};

export default GeneralSettings;
