import { useApp } from '@/context/AppProvider';
import type { UsageSummary } from '@/api';

export interface UseChatResult {
  messages: ReturnType<typeof useApp>['messages'];
  send: ReturnType<typeof useApp>['send'];
  cancel: ReturnType<typeof useApp>['cancel'];
  isStreaming: boolean;
  error: string | null;
  usage: UsageSummary | null;
  model: string;
  sessionId: string | null;
  ready: boolean;
}

export function useChat(): UseChatResult {
  const app = useApp();
  return {
    messages: app.messages,
    send: app.send,
    cancel: app.cancel,
    isStreaming: app.isStreaming,
    error: app.chatError ?? app.initError,
    usage: app.usage,
    model: app.model,
    sessionId: app.activeSessionId,
    ready: app.ready,
  };
}
