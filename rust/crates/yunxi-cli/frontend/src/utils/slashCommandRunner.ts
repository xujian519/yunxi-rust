import { api, hasBackendTools, isTauriRuntime } from '@/api';
import type { SlashExecuteResult, UsageSummary } from '@/api';
import { parsePatentSearchResults } from '@/utils/patentSearchParse';

export type SlashHandleResult = SlashExecuteResult | null;

/** 执行聊天 slash 命令；非 slash 返回 null */
export async function runSlashCommand(
  text: string,
  sessionId: string | null,
  model: string,
  _usage: UsageSummary | null,
  workspaceRoot?: string,
): Promise<SlashHandleResult> {
  const trimmed = text.trim();
  if (!trimmed.startsWith('/')) return null;

  if (isTauriRuntime() && sessionId) {
    try {
      const result = await api.executeSlashCommand(
        sessionId,
        trimmed,
        model,
        workspaceRoot,
      );
      return result ?? null;
    } catch (e) {
      return {
        kind: 'message',
        content: `命令执行失败：${e instanceof Error ? e.message : String(e)}`,
      };
    }
  }

  // Mock / 浏览器预览：保留最小命令集
  const [cmd, ...rest] = trimmed.slice(1).split(/\s+/);
  const arg = rest.join(' ').trim();

  switch (cmd.toLowerCase()) {
    case 'help':
      return {
        kind: 'message',
        content:
          '可用命令：/help /status /cost /search /analyze /doctor /compact /memory /config\n' +
          '完整命令集请在 Tauri 桌面端使用。',
      };
    case 'search':
      if (!arg) {
        return { kind: 'message', content: '用法：/search <检索关键词>' };
      }
      if (hasBackendTools()) {
        try {
          const raw = await api.patentSearch(arg, 8);
          const { items, unavailable, error } = parsePatentSearchResults(raw);
          if (unavailable) return { kind: 'message', content: `**专利检索**\n\n${unavailable}` };
          if (error && items.length === 0) {
            return { kind: 'message', content: `**专利检索**\n\n${error}` };
          }
          const lines = items
            .slice(0, 8)
            .map((r, i) => `${i + 1}. **${r.title}** — ${r.number}（${r.applicant}）`)
            .join('\n');
          return {
            kind: 'message',
            content: `**专利检索：${arg}**（${items.length} 条）\n\n${lines || '无结果'}`,
          };
        } catch (e) {
          return {
            kind: 'message',
            content: `检索失败：${e instanceof Error ? e.message : String(e)}`,
          };
        }
      }
      return { kind: 'message', content: `（Mock）检索「${arg}」` };
    case 'analyze':
      if (!arg) return { kind: 'message', content: '用法：/analyze <待分析文本>' };
      if (hasBackendTools()) {
        try {
          const raw = await api.knowledgeSearch(arg);
          return {
            kind: 'message',
            content: `**知识库分析：${arg}**\n\n${raw.length > 3000 ? `${raw.slice(0, 3000)}…` : raw}`,
          };
        } catch (e) {
          return {
            kind: 'message',
            content: `分析失败：${e instanceof Error ? e.message : String(e)}`,
          };
        }
      }
      return { kind: 'message', content: `（Mock）分析「${arg}」` };
    default:
      return null;
  }
}
