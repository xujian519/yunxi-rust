import type { FC } from 'react';
import { Image } from 'lucide-react';

const DrawingsPlaceholder: FC = () => (
  <div
    className="flex h-full flex-col items-center justify-center"
    style={{ backgroundColor: 'var(--bg-surface)', gap: 12 }}
  >
    <Image size={40} style={{ color: 'var(--text-tertiary)' }} />
    <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>附图预览（即将支持）</p>
  </div>
);

export default DrawingsPlaceholder;
