import type { StreamEvent } from './types';

const serverUrl = (): string => {
  const base = import.meta.env.VITE_SERVER_URL as string | undefined;
  return (base ?? 'http://127.0.0.1:8765').replace(/\/$/, '');
};

function wsUrl(): string {
  return `${serverUrl().replace(/^http/, 'ws')}/api/chat`;
}

function parseStreamEvent(msg: Record<string, unknown>): StreamEvent | null {
  switch (msg.type) {
    case 'text_delta':
      return { type: 'text_delta', content: String(msg.content ?? '') };
    case 'reasoning_delta':
      return { type: 'reasoning_delta', content: String(msg.content ?? '') };
    case 'tool_use':
      return {
        type: 'tool_use',
        id: String(msg.id ?? ''),
        name: String(msg.name ?? ''),
        input: String(msg.input ?? ''),
      };
    case 'tool_result':
      return {
        type: 'tool_result',
        id: String(msg.id ?? ''),
        output: String(msg.output ?? ''),
        is_error: Boolean(msg.is_error),
      };
    case 'usage':
      return {
        type: 'usage',
        input_tokens: Number(msg.input_tokens ?? 0),
        output_tokens: Number(msg.output_tokens ?? 0),
      };
    case 'permission_request':
      return {
        type: 'permission_request',
        request_id: String(msg.request_id ?? ''),
        tool: String(msg.tool ?? ''),
        input: String(msg.input ?? ''),
      };
    case 'message_stop':
      return { type: 'message_stop' };
    case 'error':
      return { type: 'error', message: String(msg.message ?? 'unknown error') };
    case 'assistant_message':
      return { type: 'text_delta', content: String(msg.content ?? '') };
    default:
      return null;
  }
}

type TurnWaiter = {
  resolve: (result: { turn_id: string; session_id: string }) => void;
  reject: (error: Error) => void;
  sessionId: string;
};

class HttpChatConnection {
  private ws: WebSocket | null = null;
  private handler: ((event: StreamEvent) => void) | null = null;
  private turnWaiter: TurnWaiter | null = null;
  private connectPromise: Promise<void> | null = null;

  setHandler(handler: ((event: StreamEvent) => void) | null) {
    this.handler = handler;
  }

  private dispatch(raw: Record<string, unknown>) {
    if (raw.type === 'connected' || raw.type === 'pong') return;

    const event = parseStreamEvent(raw);
    if (!event) return;

    this.handler?.(event);

    if (event.type === 'message_stop' && this.turnWaiter) {
      const waiter = this.turnWaiter;
      this.turnWaiter = null;
      waiter.resolve({
        turn_id: `turn-${Date.now()}`,
        session_id: waiter.sessionId,
      });
    } else if (event.type === 'error' && this.turnWaiter) {
      const waiter = this.turnWaiter;
      this.turnWaiter = null;
      waiter.reject(new Error(event.message));
    }
  }

  async connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) return;
    if (this.connectPromise) return this.connectPromise;

    this.connectPromise = new Promise<void>((resolve, reject) => {
      const ws = new WebSocket(wsUrl());
      this.ws = ws;

      ws.onopen = () => {
        ws.send(JSON.stringify({ type: 'ping' }));
        resolve();
      };

      ws.onerror = () => {
        this.connectPromise = null;
        reject(new Error('WebSocket 连接失败'));
      };

      ws.onclose = () => {
        this.ws = null;
        this.connectPromise = null;
        if (this.turnWaiter) {
          this.turnWaiter.reject(new Error('WebSocket 已断开'));
          this.turnWaiter = null;
        }
      };

      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(String(ev.data)) as Record<string, unknown>;
          this.dispatch(msg);
        } catch {
          /* ignore */
        }
      };
    });

    return this.connectPromise;
  }

  async sendTurn(
    sessionId: string,
    content: string,
    model?: string,
  ): Promise<{ turn_id: string; session_id: string }> {
    await this.connect();
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket 未连接');
    }
    if (this.turnWaiter) {
      throw new Error('已有进行中的对话轮次');
    }

    return new Promise((resolve, reject) => {
      this.turnWaiter = { resolve, reject, sessionId };
      this.ws!.send(
        JSON.stringify({
          type: 'user_message',
          session_id: sessionId,
          content,
          model,
        }),
      );
    });
  }

  close() {
    this.turnWaiter = null;
    this.connectPromise = null;
    this.ws?.close();
    this.ws = null;
  }

  sendPermissionRespond(requestId: string, outcome: string) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(
        JSON.stringify({
          type: 'permission_respond',
          request_id: requestId,
          outcome,
        }),
      );
    }
  }
}

let shared: HttpChatConnection | null = null;

function connection(): HttpChatConnection {
  if (!shared) shared = new HttpChatConnection();
  return shared;
}

export async function httpOnStream(
  _sessionId: string,
  handler: (event: StreamEvent) => void,
): Promise<() => void> {
  const conn = connection();
  conn.setHandler(handler);
  await conn.connect();
  return () => {
    conn.setHandler(null);
    conn.close();
    shared = null;
  };
}

export async function httpChatSend(
  sessionId: string,
  content: string,
  model?: string,
): Promise<{ turn_id: string; session_id: string }> {
  return connection().sendTurn(sessionId, content, model);
}

export function httpPermissionRespond(requestId: string, outcome: string) {
  connection().sendPermissionRespond(requestId, outcome);
}
