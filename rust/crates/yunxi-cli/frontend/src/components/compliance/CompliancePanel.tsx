import { useState, useEffect } from 'react';
import { tauriApi } from '../../api/tauri';

interface RuleCheckResult {
  rule_id: string;
  rule_name: string;
  passed: boolean;
  severity: string;
  action: string;
  legal_basis: string;
  details: any;
}

interface ComplianceResult {
  passed: boolean;
  total: number;
  failed: number;
  results: RuleCheckResult[];
}

export default function CompliancePanel() {
  const [text, setText] = useState('');
  const [ruleTypes, setRuleTypes] = useState<string[]>([]);
  const [context, setContext] = useState('');
  const [availableRuleTypes, setAvailableRuleTypes] = useState<string[]>([]);
  const [isChecking, setIsChecking] = useState(false);
  const [result, setResult] = useState<ComplianceResult | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    tauriApi
      .listRuleTypes()
      .then(setAvailableRuleTypes)
      .catch(setError);
  }, []);

  const toggleRuleType = (ruleType: string) => {
    if (ruleTypes.includes(ruleType)) {
      setRuleTypes(ruleTypes.filter((t) => t !== ruleType));
    } else {
      setRuleTypes([...ruleTypes, ruleType]);
    }
  };

  const runCheck = async () => {
    setIsChecking(true);
    setResult(null);
    setError('');

    try {
      const resultJson = await tauriApi.checkCompliance(
        text,
        ruleTypes.length > 0 ? ruleTypes : undefined,
        context || undefined,
      );
      const complianceResult = JSON.parse(resultJson) as ComplianceResult;
      setResult(complianceResult);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsChecking(false);
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'text-red-600';
      case 'major':
        return 'text-orange-600';
      case 'minor':
        return 'text-yellow-600';
      default:
        return 'text-gray-600';
    }
  };

  const getActionLabel = (action: string) => {
    switch (action) {
      case 'Block':
        return '阻止提交';
      case 'Warn':
        return '警告';
      case 'Review':
        return '需要审查';
      case 'Enforce':
        return '强制执行';
      case 'Log':
        return '记录日志';
      default:
        return action;
    }
  };

  return (
    <div className="p-6 bg-white dark:bg-gray-900 rounded-lg shadow-lg">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          合规检查面板
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          基于专利法规则的合规性检查（20+ 规则类型）
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
            待检查文本
          </label>
          <textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="请输入需要检查的文本（如权利要求书、说明书片段等）..."
            className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
            rows={6}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            适用场景（上下文）
          </label>
          <input
            type="text"
            value={context}
            onChange={(e) => setContext(e.target.value)}
            placeholder="如：撰写、审查、答复等..."
            className="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            规则类型（可选）
          </label>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
            {availableRuleTypes.map((ruleType) => (
              <button
                key={ruleType}
                onClick={() => toggleRuleType(ruleType)}
                className={`px-3 py-2 rounded-md text-xs font-medium transition-colors ${
                  ruleTypes.includes(ruleType)
                    ? 'bg-blue-600 text-white hover:bg-blue-700'
                    : 'bg-gray-200 text-gray-800 dark:bg-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600'
                }`}
              >
                {ruleType}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="flex justify-between items-center mb-6">
        <button
          onClick={runCheck}
          disabled={!text.trim() || isChecking}
          className="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed font-medium"
        >
          {isChecking ? '检查中...' : '执行检查'}
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
              检查结果
            </h3>
            <div
              className={`p-4 rounded-lg ${
                result.passed
                  ? 'bg-green-50 dark:bg-green-900/20'
                  : 'bg-red-50 dark:bg-red-900/20'
              }`}
            >
              <div className="flex items-center gap-3">
                <div className={`text-2xl ${result.passed ? 'text-green-600' : 'text-red-600'}`}>
                  {result.passed ? '✓' : '✗'}
                </div>
                <div>
                  <p className="text-lg font-medium text-gray-900 dark:text-white">
                    {result.passed ? '所有检查通过' : '发现违规'}
                  </p>
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    {result.total} 条规则中，{result.failed} 条未通过
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
              详细结果
            </h3>
            <div className="space-y-3">
              {result.results.map((check, index) => (
                <div
                  key={index}
                  className={`border rounded-lg p-4 ${
                    check.passed
                      ? 'border-gray-200 dark:border-gray-700'
                      : 'border-red-200 dark:border-red-700 bg-red-50/10 dark:bg-red-900/10'
                  }`}
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <div className={`text-xl ${check.passed ? 'text-green-600' : 'text-red-600'}`}>
                          {check.passed ? '✓' : '✗'}
                        </div>
                        <div>
                          <h4 className="font-medium text-gray-900 dark:text-white">
                            {check.rule_name}
                          </h4>
                          <p className="text-xs text-gray-500 dark:text-gray-400 mt-0">
                            ID: {check.rule_id}
                          </p>
                        </div>
                      </div>
                    </div>
                    <div className="text-xs">
                      <span
                        className={`font-medium ${getSeverityColor(check.severity)}`}
                      >
                        {check.severity.toUpperCase()}
                      </span>
                      {' '}
                      <span className="text-gray-600 dark:text-gray-400">
                        · {getActionLabel(check.action)}
                      </span>
                    </div>
                  </div>

                  <div className="mb-2">
                    <p className="text-sm text-gray-700 dark:text-gray-300">
                      <span className="font-medium">法律依据：</span>
                      {check.legal_basis}
                    </p>
                  </div>

                  {check.details && (
                    <div className="bg-white dark:bg-gray-800 p-3 rounded-md">
                      <details>
                        <summary className="text-sm font-medium text-blue-600 dark:text-blue-400 cursor-pointer">
                          查看详细信息
                        </summary>
                        <pre className="text-xs text-gray-700 dark:text-gray-300 mt-2 whitespace-pre-wrap">
                          {JSON.stringify(check.details, null, 2)}
                        </pre>
                      </details>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}