import { Component, type ErrorInfo, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/** 捕获渲染错误，避免整页白屏 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[YunXi UI]', error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            padding: 24,
            fontFamily: 'system-ui, sans-serif',
            color: '#c0392b',
            background: '#1a1a1a',
            minHeight: '100vh',
          }}
        >
          <h2 style={{ marginBottom: 8 }}>界面加载失败</h2>
          <pre style={{ whiteSpace: 'pre-wrap', fontSize: 13, color: '#eee' }}>
            {this.state.error.message}
          </pre>
          <p style={{ marginTop: 16, fontSize: 12, color: '#999' }}>
            请打开开发者工具 (WebInspector) 查看完整堆栈，或重启应用。
          </p>
        </div>
      );
    }
    return this.props.children;
  }
}
