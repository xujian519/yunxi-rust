import { useEffect, useState } from 'react';
import type { FC } from 'react';
import { motion } from 'framer-motion';
import {
  Github,
  FileText,
  AlertCircle,
  ExternalLink,
  CheckCircle2,
  Circle,
  Loader2,
} from 'lucide-react';
import { api } from '@/api';

const REPO_URL = 'https://github.com/xujian519/yunxi-rust';

type RoadmapStatus = 'done' | 'progress' | 'planned';

interface RoadmapItem {
  id: string;
  title: string;
  description: string;
  status: RoadmapStatus;
  eta?: string;
}

/** 桌面端短期计划（约 2–4 周） */
const SHORT_TERM_ROADMAP: RoadmapItem[] = [
  {
    id: 'p0-tauri',
    title: 'Tauri 桌面壳 + 前端构建',
    description: 'dist 打包、WebView 兼容、TitleBar / StatusBar 接入',
    status: 'done',
  },
  {
    id: 'p1-ipc-core',
    title: '核心 IPC 链路',
    description: '会话、案件、设置读写、流式对话、费用统计',
    status: 'done',
  },
  {
    id: 'p2-settings',
    title: '设置页完善',
    description: '全宽布局、模型/外观/费用与后端同步、关于页信息',
    status: 'progress',
    eta: '本周',
  },
  {
    id: 'p3-views',
    title: '中心视图真实数据',
    description: '检索、审查意见分析、对比矩阵、说明书撰写接 Rust 工具',
    status: 'planned',
    eta: '2–3 周',
  },
  {
    id: 'p4-tools-ui',
    title: '工具调用与权限 UI',
    description: 'PermissionModal、ToolCallCard、Reasoning 折叠块',
    status: 'planned',
    eta: '2–3 周',
  },
  {
    id: 'p5-onboarding',
    title: '首次启动向导',
    description: 'API Key 配置、模型选择，替代 Login Mock',
    status: 'planned',
    eta: '3–4 周',
  },
  {
    id: 'p6-release',
    title: 'macOS 打包与 CI',
    description: 'dmg 签名、自动构建、设计文档 Phase 4 验收',
    status: 'planned',
    eta: '4 周+',
  },
];

const containerVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.08 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 12 },
  show: { opacity: 1, y: 0, transition: { duration: 0.3, ease: 'easeOut' as const } },
};

function statusIcon(status: RoadmapStatus) {
  switch (status) {
    case 'done':
      return <CheckCircle2 size={16} style={{ color: 'var(--accent-primary)' }} />;
    case 'progress':
      return <Loader2 size={16} className="animate-spin" style={{ color: 'var(--accent-primary)' }} />;
    default:
      return <Circle size={16} style={{ color: 'var(--text-tertiary)' }} />;
  }
}

function statusLabel(status: RoadmapStatus): string {
  switch (status) {
    case 'done':
      return '已完成';
    case 'progress':
      return '进行中';
    default:
      return '计划中';
  }
}

function openExternal(url: string) {
  window.open(url, '_blank', 'noopener,noreferrer');
}

const AboutSettings: FC = () => {
  const [version, setVersion] = useState('…');
  const [workspaceRoot, setWorkspaceRoot] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const v = await api.getVersion();
        if (!cancelled) setVersion(v);
      } catch {
        if (!cancelled) setVersion('0.1.0');
      }
      try {
        const info = await api.getWorkspaceInfo();
        if (!cancelled) setWorkspaceRoot(info.workspaceRoot);
      } catch {
        /* mock 或 Web 预览 */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const links = [
    {
      label: '文档',
      href: `${REPO_URL}/blob/main/LOCAL_SETUP.md`,
      icon: <FileText size={12} />,
    },
    {
      label: 'GitHub',
      href: REPO_URL,
      icon: <Github size={12} />,
    },
    {
      label: '反馈问题',
      href: `${REPO_URL}/issues`,
      icon: <AlertCircle size={12} />,
    },
    {
      label: '设计规范',
      href: `${REPO_URL}/blob/main/docs/superpowers/specs/2026-05-30-yunxi-desktop-frontend-design.md`,
      icon: <ExternalLink size={12} />,
    },
  ];

  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="show"
      className="flex flex-col"
      style={{ padding: '32px 40px', maxWidth: 920, width: '100%' }}
    >
      <div
        className="flex flex-col lg:flex-row"
        style={{ gap: 40, alignItems: 'flex-start' }}
      >
        {/* 品牌信息 */}
        <div className="flex flex-col items-center lg:items-start shrink-0" style={{ width: '100%', maxWidth: 320 }}>
          <motion.div
            variants={itemVariants}
            initial={{ scale: 0.92, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ duration: 0.35, ease: [0.34, 1.56, 0.64, 1] as [number, number, number, number] }}
          >
            <img
              src="./app-icon.png"
              alt="云熙智能体"
              style={{
                width: 88,
                height: 88,
                borderRadius: 18,
                boxShadow: '0 4px 20px rgba(0,0,0,0.12)',
                objectFit: 'cover',
              }}
            />
          </motion.div>

          <motion.h2
            variants={itemVariants}
            style={{
              fontSize: 22,
              fontWeight: 600,
              color: 'var(--text-primary)',
              letterSpacing: '-0.015em',
              marginTop: 16,
            }}
          >
            云熙智能体
          </motion.h2>

          <motion.span
            variants={itemVariants}
            style={{
              fontSize: 12,
              color: 'var(--text-secondary)',
              marginTop: 4,
              fontFamily: 'JetBrains Mono, monospace',
            }}
          >
            v{version}
          </motion.span>

          <motion.p
            variants={itemVariants}
            style={{
              fontSize: 12,
              color: 'var(--text-secondary)',
              lineHeight: 1.65,
              marginTop: 12,
            }}
          >
            基于 Rust 构建的专业专利智能体，提供检索、分析、撰写与对话辅助。桌面端采用 Tauri 2 +
            React，与 YunXi runtime 深度集成。
          </motion.p>

          {workspaceRoot && (
            <motion.p
              variants={itemVariants}
              style={{
                fontSize: 10,
                color: 'var(--text-tertiary)',
                marginTop: 10,
                fontFamily: 'JetBrains Mono, monospace',
                wordBreak: 'break-all',
                lineHeight: 1.5,
              }}
            >
              工作区：{workspaceRoot}
            </motion.p>
          )}

          <motion.div
            variants={itemVariants}
            className="flex flex-wrap items-center gap-1"
            style={{ marginTop: 16 }}
          >
            {links.map((link, idx) => (
              <span key={link.label} className="flex items-center">
                <button
                  type="button"
                  onClick={() => openExternal(link.href)}
                  className="flex items-center gap-1 px-2 py-1 transition-colors"
                  style={{
                    fontSize: 12,
                    color: 'var(--accent-primary)',
                    background: 'none',
                    border: 'none',
                    cursor: 'pointer',
                    borderRadius: 4,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.textDecoration = 'underline';
                    e.currentTarget.style.backgroundColor = 'var(--accent-primary-muted)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.textDecoration = 'none';
                    e.currentTarget.style.backgroundColor = 'transparent';
                  }}
                >
                  {link.icon}
                  {link.label}
                </button>
                {idx < links.length - 1 && (
                  <span style={{ color: 'var(--text-tertiary)', fontSize: 12, margin: '0 2px' }}>
                    ·
                  </span>
                )}
              </span>
            ))}
          </motion.div>

          <motion.span
            variants={itemVariants}
            style={{
              fontSize: 11,
              color: 'var(--text-tertiary)',
              marginTop: 16,
            }}
          >
            Rust · React · Tauri 2
          </motion.span>

          <motion.span
            variants={itemVariants}
            style={{ fontSize: 11, color: 'var(--text-tertiary)', marginTop: 6 }}
          >
            &copy; 2026 YunXi Agent · MIT License
          </motion.span>
        </div>

        {/* 短期计划 */}
        <motion.div variants={itemVariants} className="flex-1 min-w-0" style={{ width: '100%' }}>
          <h3
            style={{
              fontSize: 15,
              fontWeight: 600,
              color: 'var(--text-primary)',
              marginBottom: 4,
            }}
          >
            短期计划
          </h3>
          <p style={{ fontSize: 12, color: 'var(--text-secondary)', marginBottom: 16, lineHeight: 1.5 }}>
            桌面客户端下一阶段交付目标（依据
            {' '}
            <button
              type="button"
              onClick={() =>
                openExternal(
                  `${REPO_URL}/blob/main/docs/superpowers/specs/2026-05-30-yunxi-desktop-frontend-design.md`,
                )
              }
              style={{
                color: 'var(--accent-primary)',
                background: 'none',
                border: 'none',
                padding: 0,
                cursor: 'pointer',
                fontSize: 12,
                textDecoration: 'underline',
              }}
            >
              桌面前端设计规范
            </button>
            ）
          </p>

          <div className="flex flex-col" style={{ gap: 10 }}>
            {SHORT_TERM_ROADMAP.map((item) => (
              <div
                key={item.id}
                style={{
                  padding: '12px 14px',
                  borderRadius: 10,
                  backgroundColor: 'var(--bg-surface)',
                  border: '1px solid var(--border-secondary)',
                }}
              >
                <div className="flex items-start gap-2.5">
                  <span style={{ marginTop: 2, flexShrink: 0 }}>{statusIcon(item.status)}</span>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-2 flex-wrap">
                      <span
                        style={{
                          fontSize: 13,
                          fontWeight: 500,
                          color: 'var(--text-primary)',
                        }}
                      >
                        {item.title}
                      </span>
                      <span
                        style={{
                          fontSize: 10,
                          color: 'var(--text-tertiary)',
                          padding: '2px 8px',
                          borderRadius: 999,
                          backgroundColor: 'var(--bg-elevated)',
                          whiteSpace: 'nowrap',
                        }}
                      >
                        {statusLabel(item.status)}
                        {item.eta ? ` · ${item.eta}` : ''}
                      </span>
                    </div>
                    <p
                      style={{
                        fontSize: 11,
                        color: 'var(--text-secondary)',
                        marginTop: 4,
                        lineHeight: 1.5,
                      }}
                    >
                      {item.description}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </motion.div>
  );
};

export default AboutSettings;
