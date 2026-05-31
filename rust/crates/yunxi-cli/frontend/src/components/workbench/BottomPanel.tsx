import type { FC } from 'react';
import { useState, useCallback, useEffect, useRef, memo } from 'react';
import OutputPanel from '@/components/workbench/OutputPanel';
import { AlertCircle, Terminal, FileOutput, X } from 'lucide-react';
import { api, isTauriRuntime } from '@/api';
import type { ShellEvent } from '@/api/types';
import { useApp } from '@/context/AppProvider';
import type { BottomPanelTab } from '@/types/workspace';

const tabs: { id: BottomPanelTab; label: string; Icon: typeof AlertCircle }[] = [
  { id: 'problems', label: '问题', Icon: AlertCircle },
  { id: 'output', label: '输出', Icon: FileOutput },
  { id: 'terminal', label: '终端', Icon: Terminal },
];

const BottomPanel: FC = () => {
  const {
    bottomPanelTab,
    setBottomPanelTab,
    panelLogs,
    panelProblems,
    problemCount,
    initError,
    chatError,
    activeWorkspaceFolder,
    terminalLines,
    appendTerminalLine,
    appendTerminalChunk,
    appendPanelLog,
    setBottomPanelVisible,
    bottomPanelHeight,
  } = useApp();

  const [terminalInput, setTerminalInput] = useState('');
  const [terminalCwd, setTerminalCwd] = useState<string | null>(null);
  const [terminalBusy, setTerminalBusy] = useState(false);
  const [ptyReady, setPtyReady] = useState(false);

  const sessionIdRef = useRef<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const cwdForSessionRef = useRef<string | null>(null);
  const effectiveCwd = terminalCwd ?? activeWorkspaceFolder?.path ?? null;
  const usePty = isTauriRuntime();

  const teardownPty = useCallback(() => {
    unlistenRef.current?.();
    unlistenRef.current = null;
    const sid = sessionIdRef.current;
    sessionIdRef.current = null;
    cwdForSessionRef.current = null;
    setPtyReady(false);
    if (sid) void api.shellSessionClose(sid);
  }, []);

  const startPty = useCallback(
    async (cwd: string) => {
      teardownPty();
      try {
        const sessionId = await api.shellSessionStart(cwd);
        sessionIdRef.current = sessionId;
        cwdForSessionRef.current = cwd;
        const unlisten = await api.onShell(sessionId, (event: ShellEvent) => {
          if (event.type === 'output') {
            appendTerminalChunk(event.data);
          } else if (event.type === 'error') {
            appendTerminalLine(event.message);
          } else if (event.type === 'exit') {
            const code =
              event.code != null ? String(event.code) : '—';
            appendTerminalLine(`\n[PTY 已退出 · code ${code}]`);
            setPtyReady(false);
          }
        });
        unlistenRef.current = unlisten;
        setPtyReady(true);
        appendTerminalLine(`PTY 已连接 · ${cwd}`);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendTerminalLine(msg);
        appendPanelLog(msg, 'error', '终端');
      }
    },
    [teardownPty, appendTerminalChunk, appendTerminalLine, appendPanelLog],
  );

  useEffect(() => {
    if (!usePty || bottomPanelTab !== 'terminal' || !effectiveCwd) {
      if (bottomPanelTab !== 'terminal') teardownPty();
      return;
    }
    if (cwdForSessionRef.current === effectiveCwd && sessionIdRef.current) {
      return;
    }
    void startPty(effectiveCwd);
    return () => {
      teardownPty();
    };
  }, [usePty, bottomPanelTab, effectiveCwd, startPty, teardownPty]);

  useEffect(() => {
    if (!usePty || !ptyReady || !sessionIdRef.current || bottomPanelTab !== 'terminal') {
      return;
    }
    const rowPx = 18;
    const rows = Math.max(8, Math.min(120, Math.floor((bottomPanelHeight - 48) / rowPx)));
    const cols = Math.max(
      40,
      Math.min(300, Math.floor((typeof window !== 'undefined' ? window.innerWidth : 1200) / 9)),
    );
    void api.shellSessionResize(sessionIdRef.current, rows, cols);
  }, [usePty, ptyReady, bottomPanelTab, bottomPanelHeight]);

  const runTerminalLine = useCallback(async () => {
    const cmd = terminalInput.trim();
    if (!cmd || terminalBusy) return;
    setTerminalInput('');

    if (cmd === 'help') {
      appendTerminalLine(
        usePty
          ? 'help | pwd | cd [路径] | clear | echo <文本> | 其它命令写入 PTY'
          : 'help | pwd | cd [路径] | clear | echo <文本> | 其它命令由 sh 单次执行',
      );
      return;
    }
    if (cmd === 'pwd') {
      appendTerminalLine(effectiveCwd ?? '(未设置工作区，请在侧栏选择文件夹)');
      return;
    }
    if (cmd === 'clear') {
      appendTerminalLine('__CLEAR__');
      return;
    }
    if (cmd.startsWith('echo ')) {
      appendTerminalLine(cmd.slice(5));
      return;
    }
    if (cmd === 'cd') {
      setTerminalCwd(activeWorkspaceFolder?.path ?? null);
      appendTerminalLine(activeWorkspaceFolder?.path ?? '(无工作区根)');
      return;
    }
    if (cmd.startsWith('cd ')) {
      const target = cmd.slice(3).trim();
      setTerminalCwd(target);
      appendTerminalLine(target);
      return;
    }

    if (!effectiveCwd) {
      appendTerminalLine('请先在资源管理器中选择工作区文件夹');
      return;
    }

    if (usePty && sessionIdRef.current && ptyReady) {
      appendTerminalLine(`$ ${cmd}`);
      try {
        await api.shellSessionWrite(sessionIdRef.current, `${cmd}\r\n`);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        appendTerminalLine(msg);
      }
      return;
    }

    appendTerminalLine(`$ ${cmd}`);
    setTerminalBusy(true);
    try {
      const result = await api.shellExec(effectiveCwd, cmd);
      if (result.stdout.trim()) appendTerminalLine(result.stdout.trimEnd());
      if (result.stderr.trim()) appendTerminalLine(result.stderr.trimEnd());
      appendTerminalLine(
        `[exit ${result.exitCode}${result.durationMs ? ` · ${result.durationMs}ms` : ''}]`,
      );
      appendPanelLog(
        `shell: ${cmd.slice(0, 80)} → exit ${result.exitCode}`,
        result.exitCode === 0 ? 'info' : 'warn',
        '终端',
      );
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      appendTerminalLine(msg);
      appendPanelLog(msg, 'error', '终端');
    } finally {
      setTerminalBusy(false);
    }
  }, [
    terminalInput,
    terminalBusy,
    effectiveCwd,
    activeWorkspaceFolder,
    appendTerminalLine,
    appendPanelLog,
    usePty,
    ptyReady,
  ]);

  const renderProblems = () => {
    const items = [
      ...panelProblems,
      ...(initError
        ? [{ id: 'init', severity: 'error' as const, message: initError, source: '初始化' }]
        : []),
      ...(chatError
        ? [{ id: 'chat', severity: 'error' as const, message: chatError, source: '对话' }]
        : []),
    ];
    if (items.length === 0) {
      return (
        <p style={{ padding: 12, fontSize: 12, color: 'var(--text-tertiary)' }}>未发现问题</p>
      );
    }
    return (
      <ul className="custom-scrollbar overflow-y-auto" style={{ padding: '4px 0' }}>
        {items.map((p) => (
          <li
            key={p.id}
            className="flex items-start gap-2"
            style={{
              padding: '6px 12px',
              fontSize: 12,
              borderBottom: '1px solid var(--border-primary)',
            }}
          >
            <AlertCircle
              size={14}
              style={{
                flexShrink: 0,
                marginTop: 2,
                color:
                  p.severity === 'error' ? 'var(--status-error)' : 'var(--status-warning)',
              }}
            />
            <div className="min-w-0 flex-1">
              <span style={{ color: 'var(--text-tertiary)', marginRight: 8 }}>{p.source}</span>
              <span style={{ color: 'var(--text-primary)' }}>{p.message}</span>
            </div>
          </li>
        ))}
      </ul>
    );
  };

  const renderOutput = () => <OutputPanel panelLogs={panelLogs} />;

  const visibleTerminal = terminalLines.filter((l) => l !== '__CLEAR__');

  const renderTerminal = () => (
    <div className="flex h-full flex-col">
      <pre
        className="custom-scrollbar m-0 flex-1 overflow-y-auto whitespace-pre-wrap"
        style={{
          padding: '8px 12px',
          fontSize: 11,
          fontFamily: 'ui-monospace, monospace',
          color: 'var(--text-secondary)',
        }}
      >
        {visibleTerminal.length === 0
          ? `云熙终端${usePty ? '（PTY）' : '（预览）'}。工作目录：${effectiveCwd ?? '未设置'}`
          : visibleTerminal.join('\n')}
      </pre>
      <div
        className="flex items-center gap-2"
        style={{
          borderTop: '1px solid var(--border-primary)',
          padding: '6px 10px',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <span
          style={{
            color: 'var(--accent-primary)',
            fontSize: 11,
            fontFamily: 'ui-monospace, monospace',
            maxWidth: '40%',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
          title={effectiveCwd ?? undefined}
        >
          {effectiveCwd ? effectiveCwd.split(/[/\\]/).pop() : '~'}
        </span>
        {usePty && (
          <span
            style={{
              fontSize: 9,
              color: ptyReady ? 'var(--accent-cyan)' : 'var(--text-tertiary)',
            }}
          >
            {ptyReady ? 'PTY' : '…'}
          </span>
        )}
        <span style={{ color: 'var(--text-tertiary)', fontSize: 12 }}>›</span>
        <input
          type="text"
          value={terminalInput}
          onChange={(e) => setTerminalInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') void runTerminalLine();
          }}
          disabled={terminalBusy}
          placeholder={terminalBusy ? '执行中…' : '输入命令…'}
          className="min-w-0 flex-1 bg-transparent focus:outline-none"
          style={{ fontSize: 12, color: 'var(--text-primary)' }}
        />
      </div>
    </div>
  );

  return (
    <div className="flex h-full flex-col" style={{ backgroundColor: 'var(--bg-surface)' }}>
      <div
        className="flex flex-shrink-0 items-center justify-between"
        style={{
          height: 35,
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        <div className="flex items-center" style={{ gap: 2, paddingLeft: 4 }}>
          {tabs.map(({ id, label, Icon }) => {
            const active = bottomPanelTab === id;
            const badge = id === 'problems' && problemCount > 0 ? problemCount : null;
            return (
              <button
                key={id}
                type="button"
                onClick={() => setBottomPanelTab(id)}
                className="flex items-center"
                style={{
                  height: 35,
                  padding: '0 10px',
                  gap: 6,
                  fontSize: 11,
                  fontWeight: active ? 600 : 400,
                  color: active ? 'var(--text-primary)' : 'var(--text-tertiary)',
                  boxShadow: active ? 'inset 0 -2px 0 var(--accent-primary)' : 'none',
                }}
              >
                <Icon size={14} />
                {label}
                {badge != null && (
                  <span
                    style={{
                      fontSize: 10,
                      padding: '0 5px',
                      borderRadius: 8,
                      backgroundColor: 'var(--status-error)',
                      color: 'var(--text-inverse)',
                    }}
                  >
                    {badge}
                  </span>
                )}
              </button>
            );
          })}
        </div>
        <button
          type="button"
          onClick={() => setBottomPanelVisible(false)}
          title="隐藏面板"
          style={{ width: 32, height: 35, color: 'var(--text-tertiary)' }}
        >
          <X size={14} />
        </button>
      </div>
      <div className="relative min-h-0 flex-1 overflow-hidden">
        <div
          className="absolute inset-0"
          style={{ display: bottomPanelTab === 'problems' ? 'block' : 'none' }}
        >
          {renderProblems()}
        </div>
        <div
          className="absolute inset-0"
          style={{ display: bottomPanelTab === 'output' ? 'block' : 'none' }}
        >
          {renderOutput()}
        </div>
        <div
          className="absolute inset-0"
          style={{ display: bottomPanelTab === 'terminal' ? 'block' : 'none' }}
        >
          {renderTerminal()}
        </div>
      </div>
    </div>
  );
};

export default memo(BottomPanel);
