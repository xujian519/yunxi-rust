import { useState, useEffect } from 'react';
import { tauriApi } from '../../api/tauri';

interface ReasoningPhase {
  phase: string;
  output: string;
  elapsed_ms: number;
}

interface ReasoningResult {
  phases: ReasoningPhase[];
  final_conclusion: string;
  hypotheses_generated: number;
  hypotheses_validated: number;
  total_elapsed_ms: number;
}

interface PipelineConfig {
  max_hypotheses: number;
  max_iterations: number;
  min_confidence: number;
}

export default function SuperReasoningPanel() {
  const [query, setQuery] = useState('');
  const [context, setContext] = useState('');
  const [phases, setPhases] = useState<string[]>([]);
  const [config, setConfig] = useState<PipelineConfig | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<ReasoningResult | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    tauriApi.getPipelineConfig().then((cfg) => setConfig(cfg as unknown as PipelineConfig)).catch(setError);
  }, []);

  const availablePhases = [
    'engagement',
    'analysis',
    'hypothesis',
    'discovery',
    'testing',
    'correction',
  ];

  const togglePhase = (phase: string) => {
    if (phases.includes(phase)) {
      setPhases(phases.filter(p => p !== phase));
    } else {
      setPhases([...phases, phase]);
    }
  };

  const runReasoning = async () => {
    setIsRunning(true);
    setResult(null);
    setError('');

    try {
      const resultJson = await tauriApi.runReasoning(
        query,
        context || undefined,
        phases,
        (config as unknown as Record<string, unknown>) || undefined,
      );
      const reasoningResult = JSON.parse(resultJson) as ReasoningResult;
      setResult(reasoningResult);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="p-6 bg-white dark:bg-gray-900 rounded-lg shadow-lg">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          超级推理面板
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          基于 6 阶段推理管道的结构化推理分析
        </p>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-200 rounded-lg">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 mb-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            查询
          </label>
          <textarea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="请输入要分析的问题..."
            className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
            rows={3}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            上下文（可选）
          </label>
          <textarea
            value={context}
            onChange={(e) => setContext(e.target.value)}
            placeholder="提供相关背景信息..."
            className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
            rows={2}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            推理阶段
          </label>
          <div className="grid grid-cols-3 gap-2">
            {availablePhases.map((phase) => (
              <button
                key={phase}
                onClick={() => togglePhase(phase)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  phases.includes(phase)
                    ? 'bg-blue-600 text-white hover:bg-blue-700'
                    : 'bg-gray-200 text-gray-800 dark:bg-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600'
                }`}
              >
                {phase}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="flex justify-between items-center mb-6">
        <button
          onClick={runReasoning}
          disabled={!query.trim() || isRunning}
          className="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed font-medium"
        >
          {isRunning ? '推理中...' : '启动推理'}
        </button>

        {result && (
          <button
            onClick={() => setResult(null)}
            className="px-4 py-3 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-200 font-medium"
          >
            清除结果
          </button>
        )}
      </div>

      {result && (
        <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
          <div className="mb-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
              推理结果
            </h3>
            <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
              <p className="text-gray-900 dark:text-gray-100 mb-3">
                <span className="font-medium">最终结论：</span>
                {result.final_conclusion}
              </p>
              <div className="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <span className="text-gray-600 dark:text-gray-400">生成假设：</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {result.hypotheses_generated}
                  </span>
                </div>
                <div>
                  <span className="text-gray-600 dark:text-gray-400">验证假设：</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {result.hypotheses_validated}
                  </span>
                </div>
                <div>
                  <span className="text-gray-600 dark:text-gray-400">总耗时：</span>
                  <span className="text-gray-900 dark:text-white font-medium">
                    {(result.total_elapsed_ms / 1000).toFixed(2)}s
                  </span>
                </div>
              </div>
            </div>
          </div>

          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
              推理过程
            </h3>
            <div className="space-y-4">
              {result.phases.map((phase, index) => (
                <div
                  key={index}
                  className="border border-gray-200 dark:border-gray-700 rounded-lg p-4"
                >
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="font-medium text-gray-900 dark:text-white">
                      阶段 {index + 1}: {phase.phase}
                    </h4>
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                      {phase.elapsed_ms}ms
                    </span>
                  </div>
                  <pre className="text-sm text-gray-700 dark:text-gray-300 bg-gray-50 dark:bg-gray-800 p-3 rounded overflow-x-auto">
                    {phase.output}
                  </pre>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}