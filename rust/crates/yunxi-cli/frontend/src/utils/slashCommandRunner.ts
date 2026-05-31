import { api, isTauriRuntime } from '@/api';
import type { UsageSummary } from '@/api';
import { parsePatentSearchResults } from '@/utils/patentSearchParse';

/** 执行聊天 slash 命令，返回回复 Markdown；非 slash 返回 null */
export async function runSlashCommand(
  text: string,
  refreshUsage: () => Promise<void>,
  model: string,
  usage: UsageSummary | null,
): Promise<string | null> {
  const trimmed = text.trim();
  if (!trimmed.startsWith('/')) return null;

  const [cmd, ...rest] = trimmed.slice(1).split(/\s+/);
  const arg = rest.join(' ').trim();

  switch (cmd.toLowerCase()) {
    case 'help':
      return (
        '可用命令：/help /status /cost /search <关键词> /analyze <文本>\n' +
        '专利检索走 /search，知识库走后端 KnowledgeSearch。'
      );
    case 'status':
      return `**状态**\n- 模型：${model}\n- Token：输入 ${usage?.input_tokens ?? 0} / 输出 ${usage?.output_tokens ?? 0}`;
    case 'cost': {
      await refreshUsage();
      return `**费用估算**\n- 累计成本：$${(usage?.estimated_cost ?? 0).toFixed(4)} USD`;
    }
    case 'search':
      if (!arg) return '用法：/search <检索关键词>（也可在「检索」视图回车搜索）';
      if (isTauriRuntime()) {
        try {
          const raw = await api.patentSearch(arg, 8);
          const { items, unavailable, error } = parsePatentSearchResults(raw);
          if (unavailable) return `**专利检索**\n\n${unavailable}`;
          if (error && items.length === 0) return `**专利检索**\n\n${error}`;
          const lines = items
            .slice(0, 8)
            .map((r, i) => `${i + 1}. **${r.title}** — ${r.number}（${r.applicant}）`)
            .join('\n');
          return `**专利检索：${arg}**（${items.length} 条）\n\n${lines || '无结果'}`;
        } catch (e) {
          return `检索失败：${e instanceof Error ? e.message : String(e)}`;
        }
      }
      return `（Mock）检索「${arg}」`;
    case 'analyze':
      if (!arg) return '用法：/analyze <待分析文本>';
      if (isTauriRuntime()) {
        try {
          const raw = await api.knowledgeSearch(arg);
          return `**知识库分析：${arg}**\n\n${raw.length > 3000 ? `${raw.slice(0, 3000)}…` : raw}`;
        } catch (e) {
          return `分析失败：${e instanceof Error ? e.message : String(e)}`;
        }
      }
      return `（Mock）分析「${arg}」`;
    default:
      return null;
  }
}
