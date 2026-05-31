import type { FC } from 'react';
import { FileText } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import type { MaterialFileEntry } from '@/api/types';

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

export interface ImportMaterialsPreview {
  caseId: string;
  caseName?: string;
  projectFolder: string;
  files: MaterialFileEntry[];
}

interface ImportMaterialsDialogProps {
  preview: ImportMaterialsPreview | null;
  loading?: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

const LIST_CAP = 40;

const ImportMaterialsDialog: FC<ImportMaterialsDialogProps> = ({
  preview,
  loading,
  onOpenChange,
  onConfirm,
}) => {
  const open = preview !== null;
  const files = preview?.files ?? [];
  const totalBytes = files.reduce((s, f) => s + f.sizeBytes, 0);
  const shown = files.slice(0, LIST_CAP);
  const overflow = files.length - shown.length;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>导入项目材料</DialogTitle>
          <DialogDescription>
            {preview?.caseName
              ? `将以下文件导入案件「${preview.caseName}」`
              : '确认将扫描到的材料导入当前案件'}
          </DialogDescription>
        </DialogHeader>
        <p
          className="truncate text-xs"
          style={{ color: 'var(--text-tertiary)' }}
          title={preview?.projectFolder}
        >
          {preview?.projectFolder}
        </p>
        <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
          共 {files.length} 个文件，合计 {formatBytes(totalBytes)}
          {overflow > 0 ? `（列表仅显示前 ${LIST_CAP} 个）` : ''}
        </p>
        <ul
          className="max-h-56 overflow-y-auto rounded-md border text-xs"
          style={{
            borderColor: 'var(--border-primary)',
            fontFamily: 'var(--editor-font-family)',
          }}
        >
          {shown.map((f) => (
            <li
              key={f.path}
              className="flex items-center gap-2 border-b px-3 py-1.5 last:border-b-0"
              style={{ borderColor: 'var(--border-secondary)' }}
            >
              <FileText size={12} style={{ color: 'var(--text-tertiary)', flexShrink: 0 }} />
              <span className="min-w-0 flex-1 truncate" title={f.path}>
                {f.name}
              </span>
              <span style={{ color: 'var(--text-tertiary)', flexShrink: 0 }}>
                {formatBytes(f.sizeBytes)}
              </span>
            </li>
          ))}
        </ul>
        <DialogFooter>
          <button
            type="button"
            className="rounded-md px-3 py-1.5 text-sm"
            style={{ color: 'var(--text-secondary)' }}
            disabled={loading}
            onClick={() => onOpenChange(false)}
          >
            取消
          </button>
          <button
            type="button"
            className="rounded-md px-3 py-1.5 text-sm text-white"
            style={{ background: 'var(--accent-primary)' }}
            disabled={loading || files.length === 0}
            onClick={onConfirm}
          >
            {loading ? '导入中…' : `导入 ${files.length} 个文件`}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default ImportMaterialsDialog;
