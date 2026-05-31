import { useEffect } from 'react';
import type { FC } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { api, isTauriRuntime } from '@/api';
import { useApp } from '@/context/AppProvider';

/** 桌面端：未配置 LLM API Key 时跳转引导页 */
const OnboardingGuard: FC = () => {
  const { ready, model } = useApp();
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    if (!isTauriRuntime() || !ready) return;
    const path = location.pathname;
    if (path === '/onboarding' || path === '/settings' || path === '/login') return;

    let cancelled = false;
    void (async () => {
      try {
        const ok = await api.llmAuthConfigured(model);
        if (!cancelled && !ok) {
          navigate('/onboarding', { replace: true });
        }
      } catch {
        // 检查失败时不阻断主界面
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [ready, model, location.pathname, navigate]);

  return null;
};

export default OnboardingGuard;
