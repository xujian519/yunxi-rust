import type { FC } from 'react'
import { FileText, FileCode } from 'lucide-react'
import { useApp } from '@/context/AppProvider'

const ViewModeToggle: FC = () => {
  const { docxMode, setDocxMode } = useApp()

  const isDocx = docxMode === 'docx'

  return (
    <div
      className="flex items-center rounded-md"
      style={{
        backgroundColor: 'var(--bg-sidebar-active)',
        border: '1px solid var(--border-primary)',
        height: 26,
        padding: 2,
      }}
    >
      <button
        type="button"
        onClick={() => setDocxMode('markdown')}
        className="flex items-center gap-1 rounded px-2 text-xs transition-colors"
        style={{
          height: 22,
          backgroundColor: !isDocx ? 'var(--bg-elevated)' : 'transparent',
          color: !isDocx ? 'var(--text-primary)' : 'var(--text-tertiary)',
          boxShadow: !isDocx ? '0 1px 2px rgba(0,0,0,0.05)' : 'none',
        }}
        title="Markdown 模式"
      >
        <FileCode size={12} />
        <span>Markdown</span>
      </button>
      <button
        type="button"
        onClick={() => setDocxMode('docx')}
        className="flex items-center gap-1 rounded px-2 text-xs transition-colors"
        style={{
          height: 22,
          backgroundColor: isDocx ? 'var(--bg-elevated)' : 'transparent',
          color: isDocx ? 'var(--text-primary)' : 'var(--text-tertiary)',
          boxShadow: isDocx ? '0 1px 2px rgba(0,0,0,0.05)' : 'none',
        }}
        title="DOCX 模式"
      >
        <FileText size={12} />
        <span>DOCX</span>
      </button>
    </div>
  )
}

export default ViewModeToggle