import type { FC, ComponentType, CSSProperties } from 'react';
import { useState, useRef, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Send,
  Command,
  CornerDownLeft,
  Zap,
  Search,
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

const commandIcons: Record<string, ComponentType<{ size?: number; style?: CSSProperties }>> = {
  help: HelpCircle,
  status: BarChart3,
  cost: BarChart3,
  compact: Zap,
  view: Search,
  search: Search,
  analyze: BarChart3,
  draft: PenLine,
};

const ChatView: FC = () => {
  const { messages, send, isStreaming, error, usage, model, ready } = useChat();
  const { budgetTotal, toggleMessageReasoning, activeCaseId } = useApp();
  const [inputValue, setInputValue] = useState('');
  const [showCommands, setShowCommands] = useState(false);
  const [selectedCommandIdx, setSelectedCommandIdx] = useState(0);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

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
    void send(text, activeCaseId ?? undefined);
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
  const renderMessage = (msg: ChatMessage, _idx: number) => {
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
          <span style={{ fontSize: 11, fontStyle: 'italic', color: 'var(--text-tertiary)' }}>
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
          transition={{ duration: 0.3, ease: [0.34, 1.56, 0.64, 1] as [number, number, number, number] }}
          className="flex justify-end"
          style={{ marginBottom: 16 }}
        >
          <div
            style={{
              maxWidth: '75%',
              padding: '12px 16px',
              borderRadius: '16px 16px 4px 16px',
              backgroundColor: 'var(--accent-primary-muted)',
              color: 'var(--text-primary)',
              fontSize: 14,
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
        transition={{ duration: 0.3, ease: [0.34, 1.56, 0.64, 1] as [number, number, number, number] }}
        className="group"
        style={{ marginBottom: 16 }}
      >
        <div style={{ display: 'flex', gap: 12 }}>
          <div
            style={{
              width: 28,
              height: 28,
              borderRadius: '50%',
              backgroundColor: 'var(--accent-primary-muted)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            <Sparkles size={14} style={{ color: 'var(--accent-primary)' }} />
            {msg.reasoning && (
              <ReasoningBlock
                content={msg.reasoning}
                expanded={msg.reasoningExpanded ?? false}
                onToggle={() => toggleMessageReasoning(msg.id)}
              />
            )}
            {msg.toolCalls && msg.toolCalls.length > 0 && (
              <div style={{ marginBottom: 8 }}>
                {msg.toolCalls.map((tc) => (
                  <ToolCallCard key={tc.id} tool={tc} />
                ))}
              </div>
            )}
            {/* Content */}
            <AIContent content={msg.content} />
            {/* Streaming indicator */}
            {msg.isStreaming && (
              <div style={{ marginTop: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
                <div
                  style={{
                    width: 6,
                    height: 6,
                    borderRadius: '50%',
                    backgroundColor: 'var(--accent-primary)',
                    animation: 'pulse 1.5s ease-in-out infinite',
                  }}
                />
                <span style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>思考中…</span>
              </div>
            )}
          </div>
        </div>
      </motion.div>
    );
  };

  return (
    <div className="flex h-full flex-col" style={{ backgroundColor: 'var(--bg-surface)' }}>
      {/* Cost Indicator */}
      <div
        className="flex-shrink-0"
        style={{
          height: 28,
          padding: '0 16px',
          backgroundColor: 'var(--bg-elevated)',
          borderBottom: '1px solid var(--border-primary)',
          display: 'flex',
          alignItems: 'center',
          gap: 12,
        }}
      >
        <div
          style={{
            flex: 1,
            height: 4,
            borderRadius: 2,
            backgroundColor: 'var(--border-secondary)',
            overflow: 'hidden',
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
        <span style={{ fontSize: 10, color: 'var(--text-tertiary)' }}>
          {ready
            ? isTauriRuntime()
              ? `已连接 · ${model}`
              : 'Mock 模式'
            : '正在初始化…'}
        </span>
      </div>

      {/* Messages Area */}
      <div className="custom-scrollbar flex-1 overflow-y-auto" style={{ padding: '20px 16%' }}>
        {messages.length === 0 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex h-full flex-col items-center justify-center"
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
              }}
              animate={{ scale: [1, 1.02, 1] }}
              transition={{ duration: 3, repeat: Infinity, ease: 'easeInOut' }}
            />
            <h3
              style={{
                fontSize: 18,
                fontWeight: 600,
                color: 'var(--text-primary)',
                marginBottom: 8,
              }}
            >
              你好！我是云熙，你的专利智能助手。
            </h3>
            <p
              style={{
                fontSize: 13,
                color: 'var(--text-secondary)',
                textAlign: 'center',
                marginBottom: 20,
                lineHeight: 1.5,
                maxWidth: 480,
              }}
            >
              我可以帮你检索、分析专利，或者辅助撰写专利文档。输入 / 查看可用命令。
            </p>
            <div className="flex flex-wrap justify-center" style={{ gap: 8 }}>
              {['检索相关专利', '分析权利要求', '生成说明书草案', '查看 /help 命令'].map((chip, idx) => (
                <motion.button
                  key={chip}
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.1 + idx * 0.05 }}
                  type="button"
                  style={{
                    padding: '8px 14px',
                    fontSize: 12,
                    color: 'var(--text-secondary)',
                    backgroundColor: 'var(--bg-elevated)',
                    border: '1px solid var(--border-primary)',
                    borderRadius: 9999,
                  }}
                >
                  {chip}
                </motion.button>
              ))}
            </div>
          </motion.div>
        )}

        {messages.length > 0 && (
          <>
            {messages.map((msg, idx) => renderMessage(msg, idx))}
            <div ref={messagesEndRef} />
          </>
        )}

        {error && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            style={{
              margin: '12px 0',
              padding: '10px 14px',
              borderRadius: 8,
              backgroundColor: 'rgba(239, 68, 68, 0.08)',
              border: '1px solid rgba(239, 68, 68, 0.2)',
              color: 'var(--status-error)',
              fontSize: 12,
            }}
          >
            {error}
          </motion.div>
        )}
      </div>

      {/* Input Area */}
      <div
        className="relative flex-shrink-0"
        style={{
          padding: '12px 16%',
          backgroundColor: 'var(--bg-elevated)',
          borderTop: '1px solid var(--border-primary)',
        }}
      >
        <AnimatePresence>
          {showCommands && (
            <motion.div
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 8 }}
              transition={{ duration: 0.15 }}
              style={{
                position: 'absolute',
                bottom: 'calc(100% + 8px)',
                left: '16%',
                right: '16%',
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
                  <button
                    key={cmd.id}
                    onClick={() => handleCommandClick(cmd.id)}
                    className="flex w-full items-center"
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
                    <span className="ml-auto" style={{ fontSize: 11, color: 'var(--text-tertiary)' }}>
                      {cmd.description}
                    </span>
                  </button>
                );
              })}
            </motion.div>
          )}
        </AnimatePresence>

        <div className="flex items-end" style={{ gap: 8 }}>
          <div
            style={{
              flex: 1,
              backgroundColor: 'var(--bg-sidebar)',
              border: '1px solid var(--border-primary)',
              borderRadius: 12,
              padding: '10px 14px',
              display: 'flex',
              alignItems: 'flex-end',
              gap: 8,
            }}
          >
            <textarea
              ref={textareaRef}
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={ready ? '输入消息，按 / 查看命令…' : '正在初始化会话…'}
              disabled={!ready || isStreaming}
              rows={1}
              style={{
                flex: 1,
                background: 'transparent',
                border: 'none',
                outline: 'none',
                resize: 'none',
                fontSize: 14,
                lineHeight: 1.5,
                color: 'var(--text-primary)',
                maxHeight: 200,
                minHeight: 22,
              }}
            />
            {inputValue.trim() && (
              <span style={{ fontSize: 10, color: 'var(--text-tertiary)', whiteSpace: 'nowrap' }}>
                <CornerDownLeft size={12} style={{ display: 'inline', verticalAlign: 'middle' }} />
              </span>
            )}
          </div>
          <button
            type="button"
            onClick={handleSend}
            disabled={!inputValue.trim() || isStreaming || !ready}
            style={{
              width: 40,
              height: 40,
              borderRadius: 12,
              backgroundColor:
                inputValue.trim() && !isStreaming && ready
                  ? 'var(--accent-primary)'
                  : 'var(--border-secondary)',
              color: inputValue.trim() && !isStreaming && ready ? '#fff' : 'var(--text-tertiary)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
              transition: 'all 0.15s',
              opacity: inputValue.trim() && !isStreaming && ready ? 1 : 0.5,
            }}
          >
            <Send size={18} />
          </button>
        </div>
      </div>
    </div>
  );
};

// AI Content renderer with basic markdown-like formatting
const AIContent: FC<{ content: string }> = ({ content }) => {
  const lines = content.split('\n');
  const elements: React.ReactNode[] = [];
  let inList = false;
  let listItems: string[] = [];

  const flushList = () => {
    if (listItems.length > 0) {
      elements.push(
        <ul key={`list-${elements.length}`} style={{ margin: '4px 0', paddingLeft: 20 }}>
          {listItems.map((item, i) => (
            <li key={i} style={{ fontSize: 14, lineHeight: 1.6, color: 'var(--text-primary)', marginBottom: 2 }}>
              {item}
            </li>
          ))}
        </ul>
      );
      listItems = [];
    }
    inList = false;
  };

  lines.forEach((line, i) => {
    const trimmed = line.trim();
    if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      inList = true;
      listItems.push(trimmed.slice(2));
    } else if (trimmed.match(/^\d+\.\s/)) {
      inList = true;
      listItems.push(trimmed.replace(/^\d+\.\s/, ''));
    } else {
      if (inList) flushList();
      if (trimmed === '') {
        elements.push(<div key={`br-${i}`} style={{ height: 8 }} />);
      } else if (trimmed.startsWith('```')) {
        // Code block indicator - skip
      } else if (trimmed.startsWith('# ')) {
        elements.push(
          <h3 key={`h1-${i}`} style={{ fontSize: 16, fontWeight: 600, margin: '8px 0', color: 'var(--text-primary)' }}>
            {trimmed.slice(2)}
          </h3>
        );
      } else if (trimmed.startsWith('## ')) {
        elements.push(
          <h4 key={`h2-${i}`} style={{ fontSize: 14, fontWeight: 600, margin: '6px 0', color: 'var(--text-primary)' }}>
            {trimmed.slice(3)}
          </h4>
        );
      } else if (trimmed.startsWith('**') && trimmed.endsWith('**')) {
        elements.push(
          <p key={`p-${i}`} style={{ fontSize: 14, lineHeight: 1.6, color: 'var(--text-primary)', margin: '2px 0' }}>
            <strong>{trimmed.slice(2, -2)}</strong>
          </p>
        );
      } else {
        elements.push(
          <p key={`p-${i}`} style={{ fontSize: 14, lineHeight: 1.6, color: 'var(--text-primary)', margin: '2px 0' }}>
            {trimmed}
          </p>
        );
      }
    }
  });

  if (inList) flushList();

  return <>{elements}</>;
};

export default ChatView;
