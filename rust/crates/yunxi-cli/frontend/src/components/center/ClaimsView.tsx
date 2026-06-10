import type { FC } from 'react';
import { useCallback } from 'react';
import { useState } from 'react';
import { useApp } from '@/context/AppProvider';
import { motion } from 'framer-motion';
import { isTauriRuntime } from '@/api';
import { tauriApi } from '@/api/tauri';
import { sampleClaims } from '@/data/mockData';

const ClaimsView: FC = () => {
  const { activeDocContent } = useApp();
  const [parseResult, setParseResult] = useState<string | null>(null);
  const [parsing, setParsing] = useState(false);
  const [parseError, setParseError] = useState<string | null>(null);
  const [activeLine, setActiveLine] = useState<number | null>(null);
  const claimsText = activeDocContent || sampleClaims;
  const lines = claimsText.split('\n');

  const handleParse = useCallback(async () => {
    if (!claimsText.trim()) return;
    setParsing(true);
    setParseError(null);
    setParseResult(null);
    try {
      if (isTauriRuntime()) {
        const result = await tauriApi.claimParse(claimsText);
        setParseResult(result);
      } else {
        setParseResult('解析完成（模拟模式）\n\n' + claimsText.split('\n').slice(0, 5).join('\n') + '\n...');
      }
    } catch (err) {
      setParseError(err instanceof Error ? err.message : String(err));
    } finally {
      setParsing(false);
    }
  }, [claimsText]);

  const renderLine = (line: string, idx: number) => {
    const isClaimNumber = line.trim().match(/^\[\d{4}\\]\s+\d+\./);
    const isActive = activeLine === idx;

    return (
      <div
        key={idx}
        className="flex transition-colors duration-100"
        style={{
          backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
          minHeight: 24,
        }}
        onClick={() => setActiveLine(idx)}
      >
        {/* Line Number */}
        <div
          className="flex-shrink-0 select-none text-right"
          style={{
            width: 48,
            paddingRight: 12,
            fontSize: 11,
            fontFamily: 'var(--editor-font-family)',
            color: 'var(--text-tertiary)',
            lineHeight: '24px',
          }}
        >
          {line.trim() ? idx + 1 : ''}
        </div>
        {/* Content */}
        <div
          className="flex-1"
          style={{
            fontSize: 13,
            fontFamily: 'var(--editor-font-family)',
            lineHeight: '24px',
            color: 'var(--text-primary)',
            paddingLeft: 8,
            whiteSpace: 'pre',
            overflow: 'visible',
          }}
        >
          {isClaimNumber ? (
            <span>
              <span style={{ color: 'var(--accent-secondary)', fontWeight: 600 }}>
                {line.match(/^\[\d{4}\]\s+(\d+\.)?/)?.[0] || line}
              </span>
              <span>{line.slice((line.match(/^\[\d{4}\]\s+\d+\./)?.[0] || '').length)}</span>
            </span>
          ) : (
            <span style={{ color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)' }}>
              {line}
            </span>
          )}
        </div>
      </div>
    );
  };
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="flex flex-col h-full"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      {/* Toolbar header */}
      <div
        className="flex items-center gap-2 px-3 py-2 flex-shrink-0 select-none"
        style={{ borderBottom: '1px solid var(--border-subtle)' }}
      >
        <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--text-primary)' }}>{claimsText.split('\n').length} 项</span>
        <div className="flex-1" />
        <button
          onClick={handleParse}
          disabled={parsing || !claimsText.trim()}
          style={{
            padding: '4px 12px',
            fontSize: 13,
            fontWeight: 500,
            borderRadius: 6,
            border: '1px solid var(--accent-primary)',
            backgroundColor: parsing ? 'var(--bg-surface)' : 'var(--accent-primary)',
            color: parsing ? 'var(--text-tertiary)' : '#fff',
            cursor: parsing || !claimsText.trim() ? 'not-allowed' : 'pointer',
            transition: 'opacity 0.15s',
          }}
        >
          {parsing ? '解析中...' : '解析'}
        </button>
      </div>

      {/* Parse results overlay */}
      {(parseResult || parseError) && (
        <div
          style={{
            padding: '10px 16px',
            fontSize: 13,
            lineHeight: '1.5',
            fontFamily: 'var(--editor-font-family)',
            whiteSpace: 'pre-wrap',
            borderBottom: '1px solid var(--border-subtle)',
            backgroundColor: parseError ? 'var(--bg-danger-subtle, #fff0f0)' : 'var(--bg-accent-subtle, #f0f8ff)',
            color: parseError ? 'var(--text-danger, #c00)' : 'var(--text-primary)',
          }}
        >
          {parseError ? `❌ ${parseError}` : parseResult}
          <button
            onClick={() => { setParseResult(null); setParseError(null); }}
            style={{
              marginLeft: 12, fontSize: 11, color: 'var(--text-tertiary)',
              cursor: 'pointer', textDecoration: 'underline', border: 'none', background: 'none',
            }}
          >
            关闭
          </button>
        </div>
      )}

      {/* Claims content */}
      <div className="flex-1 overflow-auto custom-scrollbar">
        <div style={{ padding: '16px 0', minWidth: 'fit-content' }}>
          {lines.map((line, idx) => renderLine(line, idx))}
        </div>
      </div>
    </motion.div>
  );
};

export default ClaimsView;
