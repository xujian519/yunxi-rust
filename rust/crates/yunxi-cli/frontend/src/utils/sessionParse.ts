import type { ChatMessage } from '@/data/mockData';

interface SessionBlock {
  type: string;
  text?: string;
  name?: string;
  input?: string;
  output?: string;
  is_error?: boolean;
}

interface SessionMessage {
  role: string;
  blocks?: SessionBlock[];
}

interface SessionJson {
  messages?: SessionMessage[];
}

function formatTime(epochSecs?: number): string {
  if (!epochSecs) return '';
  const date = new Date(epochSecs * 1000);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  }
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (date.toDateString() === yesterday.toDateString()) {
    return '昨天';
  }
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
}

function blocksToText(blocks: SessionBlock[] | undefined): string {
  if (!blocks?.length) return '';
  return blocks
    .map((block) => {
      if (block.type === 'text' && block.text) return block.text;
      if (block.type === 'reasoning' && block.text) return block.text;
      if (block.type === 'tool_use') {
        return `\n🔧 **${block.name ?? 'tool'}**\n\`\`\`\n${block.input ?? ''}\n\`\`\``;
      }
      if (block.type === 'tool_result') {
        const prefix = block.is_error ? '❌' : '✅';
        return `\n${prefix} ${block.output ?? ''}`;
      }
      return '';
    })
    .join('')
    .trim();
}

/** 将 runtime Session JSON 转为 UI 消息列表 */
export function parseSessionToMessages(sessionJson: string, modifiedAt?: number): ChatMessage[] {
  let data: SessionJson;
  try {
    data = JSON.parse(sessionJson) as SessionJson;
  } catch {
    return [];
  }

  const ts = formatTime(modifiedAt);
  const result: ChatMessage[] = [];

  for (const [idx, msg] of (data.messages ?? []).entries()) {
    const content = blocksToText(msg.blocks);
    if (!content) continue;
    if (content.startsWith('[session title:')) continue;

    const role =
      msg.role === 'user' ? 'user' : msg.role === 'assistant' ? 'ai' : ('system' as const);

    result.push({
      id: `sess-${idx}-${role}`,
      role,
      content,
      timestamp: ts,
    });
  }

  return result;
}

/** 从会话 JSON 提取标题 */
export function sessionTitleFromJson(sessionJson: string, fallbackId: string): string {
  try {
    const data = JSON.parse(sessionJson) as SessionJson;
    for (const msg of data.messages ?? []) {
      const text = blocksToText(msg.blocks);
      if (text.startsWith('[session title:')) {
        return text.replace('[session title:', '').replace(']', '').trim();
      }
      if (msg.role === 'user' && text.length > 0) {
        return text.length > 24 ? `${text.slice(0, 24)}…` : text;
      }
    }
  } catch {
    // ignore
  }
  return fallbackId.slice(0, 16);
}

export { formatTime as formatSessionTime };
