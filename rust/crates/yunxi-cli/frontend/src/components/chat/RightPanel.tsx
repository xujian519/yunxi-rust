import type { FC, ComponentType, CSSProperties } from 'react';
import { useState, useRef, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Send,
  Paperclip,
  Command,
  CornerDownLeft,
  Zap,
  Search,
  FileText,
  BarChart3,
  PenLine,
  HelpCircle,
  Sparkles,
} from 'lucide-react';
import { slashCommands } from '@/data/mockData';
import type { ChatMessage } from '@/data/mockData';
import { useChat } from '@/hooks/useChat';
import { useApp } from '@/context/AppProvider';
import { isTauriRuntime } from '@/api';
import ReasoningBlock from '@/components/chat/ReasoningBlock';
import ToolCallCard from '@/components/chat/ToolCallCard';

interface RightPanelProps {
  width?: number;
  onClose?: () => void;
}

const commandIcons: Record<string, ComponentType<{ size?: number; style?: CSSProperties }>> = {
  help: HelpCircle,
  status: BarChart3,
  cost: Zap,
  compact: CornerDownLeft,
  view: FileText,
  search: Search,
  analyze: Sparkles,
  draft: PenLine,
};

const RightPanel: FC<RightPanelProps> = ({ onClose }) => {
  const { messages, send, isStreaming, error, usage, model, ready } = useChat();
  const { budgetTotal, toggleMessageReasoning } = useApp();
  const [inputValue, setInputValue] = useState('');
  const [showCommands, setShowCommands] = useState(false);
  const [selectedCommandIdx, setSelectedCommandIdx] = useState(0);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const inputContainerRef = useRef<HTMLDivElement>(null);

  const costUsed = usage?.estimated_cost ?? 0;
  const costTotal = budgetTotal;
  const costPercent = (costUsed / costTotal) * 100;

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      const maxHeight = 200;
      const scrollHeight = Math.min(textareaRef.current.scrollHeight, maxHeight);
      textareaRef.current.style.height = scrollHeight + 'px';
    }
  }, [inputValue]);

  useEffect(() => {
    if (inputValue === '/') {
      setShowCommands(true);
      setSelectedCommandIdx(0);
    } else if (!inputValue.startsWith('/')) {
      setShowCommands(false);
    } else if (inputValue.startsWith('/')) {
      const query = inputValue.slice(1).toLowerCase();
      const hasMatch = slashCommands.some(
        (c) => c.label.toLowerCase().includes(query) || c.id.toLowerCase().includes(query)
      );
      setShowCommands(hasMatch);
    }
  }, [inputValue]);

  const handleSend = () => {
    if (!inputValue.trim() || isStreaming || !ready) return;

    const text = inputValue.trim();
    setInputValue('');
    setShowCommands(false);
    void send(text);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (showCommands) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedCommandIdx((prev) => (prev + 1) % slashCommands.length);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedCommandIdx((prev) => (prev - 1 + slashCommands.length) % slashCommands.length);
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        const cmd = slashCommands[selectedCommandIdx];
        setInputValue(`/${cmd.id} `);
        setShowCommands(false);
        return;
      }
      if (e.key === 'Escape') {
        setShowCommands(false);
        return;
      }
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleCommandClick = (cmdId: string) => {
    setInputValue(`/${cmdId} `);
    setShowCommands(false);
    textareaRef.current?.focus();
  };

  const renderMessage = (msg: ChatMessage, idx: number) => {
    if (msg.role === 'system') {
      return (
        <motion.div
          key={msg.id}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.2 }}
          className="text-center"
          style={{ padding: '4px 0' }}
        >
          <span
            style={{
              fontSize: 11,
              fontStyle: 'italic',
              color: 'var(--text-tertiary)',
            }}
          >
            {msg.content}
          </span>
        </motion.div>
      );
    }

    if (msg.role === 'user') {
      return (
        <motion.div
          key={msg.id}
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{
            duration: 0.3,
            ease: [0.34, 1.56, 0.64, 1] as [number, number, number, number],
          }}
          className="flex justify-end"
          style={{ marginBottom: 12 }}
        >
          <div
            style={{
              maxWidth: '85%',
              padding: '10px 14px',
              borderRadius: '12px 12px 4px 12px',
              backgroundColor: 'var(--accent-primary-muted)',
              color: 'var(--text-primary)',
              fontSize: 13,
              lineHeight: 1.6,
            }}
          >
            {msg.content}
          </div>
        </motion.div>
      );
    }

    // AI message
    return (
      <motion.div
        key={msg.id}
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{
          duration: 0.3,
          ease: [0.34, 1.56, 0.64, 1] as [number, number, number, number],
          delay: idx * 0.05,
        }}
        className="group"
        style={{ marginBottom: 12, paddingLeft: 12 }}
      >
        <div
          style={{
            borderLeft: msg.isStreaming
              ? '2px solid var(--accent-cyan)'
              : '2px solid transparent',
            padding: '8px 0 8px 10px',
          }}
        >
          {(msg.reasoning || (msg.isStreaming && msg.reasoning !== undefined)) && (
            <ReasoningBlock
              content={msg.reasoning ?? ''}
              expanded={msg.reasoningExpanded ?? false}
              isStreaming={msg.isStreaming}
              onToggle={() => toggleMessageReasoning(msg.id)}
            />
          )}
          {msg.toolCalls?.map((tool) => (
            <ToolCallCard key={tool.id} tool={tool} />
          ))}
          <div
            style={{
              fontSize: 13,
              lineHeight: 1.7,
              color: 'var(--text-primary)',
              whiteSpace: 'pre-wrap',
            }}
          >
            {msg.isStreaming ? (
              <>
                {msg.content ? <AIContent content={msg.content} /> : null}
                <span
                  className="ml-0.5 inline-block"
                  style={{
                    width: 2,
                    height: '1em',
                    backgroundColor: 'var(--accent-cyan)',
                    verticalAlign: 'text-bottom',
                    animation: 'blink 1s step-end infinite',
                  }}
                />
              </>
            ) : msg.content ? (
              <AIContent content={msg.content} />
            ) : null}
          </div>
        </div>
        {/* Message Actions (hover) */}
        {!msg.isStreaming && (
          <div
            className="flex items-center opacity-0 transition-opacity duration-100 group-hover:opacity-100"
            style={{ gap: 8, paddingLeft: 10, marginTop: 4 }}
          >
            <ActionButton icon={<CopyIcon size={12} />} />
            <ActionButton icon={<RefreshIcon size={12} />} />
            <ActionButton icon={<ThumbsUpIcon size={12} />} />
            <ActionButton icon={<ThumbsDownIcon size={12} />} />
          </div>
        )}
      </motion.div>
    );
  };

  return (
    <div
      className="flex h-full flex-col"
      style={{
        backgroundColor: 'var(--bg-sidebar)',
        backdropFilter: 'blur(20px) saturate(1.15)',
        borderLeft: '1px solid var(--border-primary)',
      }}
    >
      <div
        className="flex flex-shrink-0 items-center justify-between"
        style={{
          height: 35,
          padding: '0 12px',
          borderBottom: '1px solid var(--border-primary)',
          fontSize: 11,
          fontWeight: 600,
          letterSpacing: '0.04em',
          textTransform: 'uppercase',
          color: 'var(--text-secondary)',
        }}
      >
        <span>AI 助手</span>
        {onClose ? (
          <button
            type="button"
            onClick={onClose}
            title="隐藏面板"
            style={{ color: 'var(--text-tertiary)', fontSize: 14, lineHeight: 1 }}
          >
            ×
          </button>
        ) : null}
      </div>

      {/* Cost Indicator Bar */}
      <div
        className="flex-shrink-0"
        style={{
          height: 32,
          padding: '0 12px',
          backgroundColor: 'var(--bg-elevated)',
          borderBottom: '1px solid var(--border-primary)',
          opacity: 0.7,
        }}
      >
        <div className="flex h-full items-center justify-between">
          <div
            style={{
              flex: 1,
              height: 4,
              borderRadius: 2,
              backgroundColor: 'var(--border-secondary)',
              overflow: 'hidden',
              marginRight: 12,
            }}
          >
            <motion.div
              initial={{ width: 0 }}
              animate={{ width: `${costPercent}%` }}
              transition={{ duration: 0.5 }}
              style={{
                height: '100%',
                borderRadius: 2,
                background: `linear-gradient(to right, var(--status-success), var(--status-warning), var(--status-error))`,
              }}
            />
          </div>
          <span
            style={{
              fontSize: 10,
              fontWeight: 500,
              color: costPercent > 80 ? 'var(--status-error)' : 'var(--text-tertiary)',
              whiteSpace: 'nowrap',
              fontVariantNumeric: 'tabular-nums',
            }}
          >
            ${costUsed.toFixed(2)} / ${costTotal.toFixed(2)}
          </span>
        </div>
      </div>

      {/* Messages Area */}
      <div className="custom-scrollbar flex-1 overflow-y-auto" style={{ padding: 16 }}>
        {/* Empty State */}
        {messages.length === 0 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex h-full flex-col items-center justify-center"
            style={{ padding: 32 }}
          >
            <motion.img
              src="./app-icon.png"
              alt="云熙"
              style={{
                width: 64,
                height: 64,
                borderRadius: '50%',
                objectFit: 'cover',
                marginBottom: 16,
              }}
              animate={{ scale: [1, 1.02, 1] }}
              transition={{ duration: 3, repeat: Infinity, ease: 'easeInOut' }}
            />
            <h3
              style={{
                fontSize: 16,
                fontWeight: 600,
                color: 'var(--text-primary)',
                marginBottom: 6,
                textAlign: 'center',
              }}
            >
              你好！我是云熙，你的专利智能助手。
            </h3>
            <p
              style={{
                fontSize: 12,
                color: 'var(--text-secondary)',
                textAlign: 'center',
                marginBottom: 16,
                lineHeight: 1.5,
              }}
            >
              我可以帮你检索、分析专利，或者辅助撰写专利文档。
            </p>
          </motion.div>
        )}

        {/* Messages */}
        {messages.map((msg, idx) => renderMessage(msg, idx))}

        {/* Typing Indicator */}
        {isStreaming && messages.every((m) => !m.isStreaming) && (
          <motion.div
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center"
            style={{ gap: 6, paddingLeft: 12, marginBottom: 12 }}
          >
            <div className="flex items-center" style={{ gap: 3 }}>
              {[0, 1, 2].map((i) => (
                <motion.div
                  key={i}
                  className="rounded-full"
                  style={{
                    width: 6,
                    height: 6,
                    backgroundColor: 'var(--accent-cyan)',
                  }}
                  animate={{ y: [0, -4, 0] }}
                  transition={{
                    duration: 0.4,
                    repeat: Infinity,
                    delay: i * 0.15,
                    ease: 'easeInOut',
                  }}
                />
              ))}
            </div>
            <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>思考中...</span>
          </motion.div>
        )}

        <div ref={messagesEndRef} />

        {error && (
          <div
            style={{
              fontSize: 11,
              color: 'var(--status-error)',
              padding: '4px 12px',
              marginBottom: 8,
            }}
          >
            {error}
          </div>
        )}
      </div>

      {/* Connection Status */}
      <div
        className="flex items-center justify-center"
        style={{
          height: 24,
          gap: 6,
          fontSize: 10,
          color: 'var(--text-tertiary)',
        }}
      >
        <div
          className="rounded-full"
          style={{
            width: 8,
            height: 8,
            backgroundColor: ready ? 'var(--status-success)' : 'var(--status-warning)',
          }}
        />
        <span>
          {ready
            ? isTauriRuntime()
              ? `已连接 · ${model}`
              : 'Mock 模式'
            : '正在初始化会话…'}
        </span>
      </div>

      {/* Chat Input Area */}
      <div
        ref={inputContainerRef}
        className="relative flex-shrink-0"
        style={{
          padding: 12,
          backgroundColor: 'var(--bg-elevated)',
          borderTop: '1px solid var(--border-primary)',
          opacity: 0.85,
        }}
      >
        {/* Slash Command Menu */}
        <AnimatePresence>
          {showCommands && (
            <motion.div
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 8 }}
              transition={{ duration: 0.15, ease: 'easeOut' }}
              style={{
                position: 'absolute',
                bottom: 'calc(100% + 8px)',
                left: 12,
                right: 12,
                backgroundColor: 'var(--bg-elevated)',
                borderRadius: 12,
                border: '1px solid var(--border-primary)',
                boxShadow: '0 12px 40px rgba(0,0,0,0.12)',
                overflow: 'hidden',
                zIndex: 60,
              }}
            >
              {slashCommands.map((cmd, idx) => {
                const Icon = commandIcons[cmd.id] || Command;
                const isSelected = idx === selectedCommandIdx;
                return (
                  <motion.button
                    key={cmd.id}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: idx * 0.02 }}
                    onClick={() => handleCommandClick(cmd.id)}
                    className="flex w-full items-center transition-colors duration-100"
                    style={{
                      height: 36,
                      padding: '8px 12px',
                      gap: 10,
                      backgroundColor: isSelected ? 'var(--accent-primary-muted)' : 'transparent',
                    }}
                    onMouseEnter={() => setSelectedCommandIdx(idx)}
                    type="button"
                  >
                    <Icon
                      size={16}
                      style={{
                        color: isSelected ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                        flexShrink: 0,
                      }}
                    />
                    <span
                      style={{
                        fontSize: 12,
                        fontWeight: isSelected ? 500 : 400,
                        color: isSelected ? 'var(--text-primary)' : 'var(--text-secondary)',
                      }}
                    >
                      {cmd.label}
                    </span>
                    <span
                      className="ml-auto"
                      style={{
                        fontSize: 11,
                        color: 'var(--text-tertiary)',
                      }}
                    >
                      {cmd.description}
                    </span>
                    {cmd.shortcut && (
                      <span
                        style={{
                          fontSize: 10,
                          fontFamily: "'JetBrains Mono', monospace",
                          color: 'var(--text-tertiary)',
                          marginLeft: 4,
                        }}
                      >
                        {cmd.shortcut}
                      </span>
                    )}
                  </motion.button>
                );
              })}
            </motion.div>
          )}
        </AnimatePresence>

        {/* Input Row */}
        <div className="flex items-end" style={{ gap: 8 }}>
          <button
            className="flex items-center justify-center flex-shrink-0 transition-colors duration-150"
            style={{
              width: 28,
              height: 28,
              borderRadius: 6,
              color: 'var(--text-tertiary)',
              marginBottom: 4,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.color = 'var(--text-secondary)';
              e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = 'var(--text-tertiary)';
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
            type="button"
          >
            <Paperclip size={16} />
          </button>

          <div
            className="relative flex-1"
            style={{
              backgroundColor: 'var(--bg-surface)',
              borderRadius: 12,
              border: '1px solid var(--border-primary)',
              transition: 'border-color 0.2s ease, box-shadow 0.2s ease',
            }}
          >
            <textarea
              ref={textareaRef}
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="输入消息或使用 / 查看命令..."
              className="w-full resize-none bg-transparent focus:outline-none"
              style={{
                padding: '10px 14px',
                fontSize: 13,
                lineHeight: 1.5,
                color: 'var(--text-primary)',
                minHeight: 40,
                maxHeight: 200,
                borderRadius: 12,
              }}
              onFocus={(e) => {
                const parent = e.currentTarget.parentElement;
                if (parent) {
                  parent.style.borderColor = 'var(--border-focus)';
                  parent.style.boxShadow = '0 0 0 3px var(--accent-primary-muted)';
                }
              }}
              onBlur={(e) => {
                const parent = e.currentTarget.parentElement;
                if (parent) {
                  parent.style.borderColor = 'var(--border-primary)';
                  parent.style.boxShadow = 'none';
                }
              }}
              rows={1}
            />
          </div>

          <button
            onClick={handleSend}
            disabled={!inputValue.trim() || isStreaming || !ready}
            className="flex items-center justify-center flex-shrink-0 transition-all duration-200"
            style={{
              width: 28,
              height: 28,
              borderRadius: '50%',
              backgroundColor:
                inputValue.trim() && !isStreaming && ready
                  ? 'var(--accent-primary)'
                  : 'var(--border-primary)',
              color:
                inputValue.trim() && !isStreaming && ready
                  ? '#FFFFFF'
                  : 'var(--text-tertiary)',
              marginBottom: 4,
            }}
            onMouseEnter={(e) => {
              if (inputValue.trim()) {
                e.currentTarget.style.transform = 'scale(1.05)';
              }
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
            }}
            type="button"
          >
            <Send size={14} />
          </button>
        </div>

        {/* Shortcut Hint */}
        <div
          className="mt-1.5 text-center"
          style={{
            fontSize: 10,
            color: 'var(--text-tertiary)',
            letterSpacing: '0.01em',
          }}
        >
          Enter 发送 · Shift+Enter 换行 · / 命令
        </div>
      </div>
    </div>
  );
};

// AI Content renderer with basic markdown-like formatting
const AIContent: FC<{ content: string }> = ({ content }) => {
  const lines = content.split('\n');

  return (
    <>
      {lines.map((line, i) => {
        // Bold: **text**
        if (line.startsWith('**') && line.endsWith('**') && line.length > 4) {
          return (
            <div key={i} style={{ fontWeight: 600, marginTop: i > 0 ? 8 : 0 }}>
              {line.slice(2, -2)}
            </div>
          );
        }
        // Numbered list
        if (line.match(/^\d+\.\s/)) {
          return (
            <div key={i} style={{ marginTop: i > 0 ? 4 : 0 }}>
              {line}
            </div>
          );
        }
        // Regular line
        return (
          <div key={i} style={{ marginTop: i > 0 ? 2 : 0 }}>
            {line || <span>&nbsp;</span>}
          </div>
        );
      })}
    </>
  );
};

// Small icon components
const CopyIcon: FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
    <rect x="3" y="3" width="6" height="6" rx="1" />
    <path d="M7.5 2.5V2a1 1 0 00-1-1H2a1 1 0 00-1 1v4.5a1 1 0 001 1h.5" />
  </svg>
);

const RefreshIcon: FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
    <path d="M2 6a4 4 0 014-4 4 4 0 012.8 1.2L10 5M10 2v3H7" />
    <path d="M10 6a4 4 0 01-4 4 4 4 0 01-2.8-1.2L2 7M2 10V7h3" />
  </svg>
);

const ThumbsUpIcon: FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
    <path d="M2 5.5v4h1.5M3.5 9.5H2a.5.5 0 01-.5-.5v-4a.5.5 0 01.5-.5h1.5L5.5 2a1 1 0 011 1v.5h1.5a2 2 0 012 2v2.5a1.5 1.5 0 01-1.5 1.5H3.5z" />
  </svg>
);

const ThumbsDownIcon: FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
    <path d="M2 6.5v-4h1.5M3.5 2.5H2a.5.5 0 00-.5.5v4a.5.5 0 00.5.5h1.5L5.5 10a1 1 0 001-1v-.5h1.5a2 2 0 002-2v-2.5A1.5 1.5 0 008.5 2.5H3.5z" />
  </svg>
);

const ActionButton: FC<{
  icon: React.ReactNode;
  onClick?: () => void;
}> = ({ icon, onClick }) => (
  <button
    onClick={onClick}
    className="flex items-center justify-center transition-colors duration-150"
    style={{
      width: 22,
      height: 22,
      borderRadius: 4,
      color: 'var(--text-tertiary)',
    }}
    onMouseEnter={(e) => {
      e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
      e.currentTarget.style.color = 'var(--text-secondary)';
    }}
    onMouseLeave={(e) => {
      e.currentTarget.style.backgroundColor = 'transparent';
      e.currentTarget.style.color = 'var(--text-tertiary)';
    }}
    type="button"
  >
    {icon}
  </button>
);

export default RightPanel;
