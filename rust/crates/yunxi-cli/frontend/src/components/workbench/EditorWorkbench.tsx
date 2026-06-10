import type { FC } from 'react';
import { lazy, Suspense, useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import ClaimsView from '@/components/center/ClaimsView';
import CompareView from '@/components/center/CompareView';
import ReviewView from '@/components/center/ReviewView';
import SearchView from '@/components/center/SearchView';
import DraftView from '@/components/center/DraftView';
import DrawingsPlaceholder from '@/components/workbench/DrawingsPlaceholder';
import WelcomeEditor from '@/components/workbench/WelcomeEditor';
import EditorTabBar from '@/components/workbench/EditorTabBar';
import { DocxEditorView } from '@/components/docx-editor';
import { useApp } from '@/context/AppProvider';
import { buildPrompt } from '@/utils/aiBridge';
import { tauriApi } from '@/api/tauri';

const PdfViewer = lazy(() => import('@/components/viewers/PdfViewer'));
const ExcelViewer = lazy(() => import('@/components/viewers/ExcelViewer'));

function ExternalFileViewer({ filePath, fileType }: { filePath: string; fileType: string }) {
  const [convertedPath, setConvertedPath] = useState<string | null>(null);
  const [converting, setConverting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (fileType === 'doc') {
      setConverting(true);
      tauriApi
        .libreofficeConvert(filePath)
        .then((pdfPath) => {
          setConvertedPath(pdfPath);
          setConverting(false);
        })
        .catch((e) => {
          setError(String(e));
          setConverting(false);
        });
    }
  }, [filePath, fileType]);

  if (fileType === 'doc' && converting) {
    return (
      <div className="flex h-full items-center justify-center" style={{ color: 'var(--text-secondary)' }}>
        正在通过 LibreOffice 转换 DOC → PDF…
      </div>
    );
  }
  if (error) {
    return (
      <div className="flex h-full items-center justify-center" style={{ color: 'var(--status-error)' }}>
        {error}
      </div>
    );
  }

  const finalPath = convertedPath ?? filePath;
  const finalType = convertedPath ? 'pdf' : fileType;

  return (
    <Suspense
      fallback={
        <div className="flex h-full items-center justify-center" style={{ color: 'var(--text-secondary)' }}>
          加载中…
        </div>
      }
    >
      {finalType === 'pdf' && <PdfViewer filePath={finalPath} />}
      {(finalType === 'xlsx' || finalType === 'xls') && <ExcelViewer filePath={finalPath} />}
      {(finalType === 'docx' || finalType === 'doc') && (
        <DocxEditorView
          markdownContent=""
          mode="viewing"
          readOnly={false}
          documentName={filePath.split('/').pop() ?? '文档'}
        />
      )}
    </Suspense>
  );
}
const EditorWorkbench: FC = () => {
  const { activeView, editorTabs, activeTabId, activeCase, activeDocId, activeDocContent, docxMode, updateCaseDocument, send } = useApp();

  const activeTab = editorTabs.find((t) => t.id === activeTabId);
  const activeDoc = activeCase?.documents.find((d) => d.id === activeDocId);
  const showDrawings =
    activeTab?.kind === 'document' && activeDoc?.type === 'drawings';

  const renderView = () => {
    if (editorTabs.length === 0) return <WelcomeEditor />;
    if (showDrawings) return <DrawingsPlaceholder />;
    // 外部文件标签页
    if (activeTab?.kind === 'external' && activeTab.filePath && activeTab.fileType) {
      return (
        <ExternalFileViewer
          filePath={activeTab.filePath}
          fileType={activeTab.fileType}
        />
      );
    }
    if (docxMode === 'docx') {
      const content = activeDoc?.contentMd || activeDocContent;
      const isReview = activeView === 'review';
      const isReadOnly = activeView === 'compare' || activeView === 'search';
      return (
        <DocxEditorView
          markdownContent={content}
          mode={isReview ? 'suggesting' : 'editing'}
          readOnly={isReadOnly}
          onChange={(markdown) => {
            if (activeDocId && !isReadOnly) void updateCaseDocument(activeDocId, markdown);
          }}
          onAIAction={(action, selectedText) => {
            const prompt = buildPrompt(action, selectedText);
            void send(prompt);
          }}
        />
      );
    }
    switch (activeView) {
      case 'claims':
        return <ClaimsView />;
      case 'compare':
        return <CompareView />;
      case 'review':
        return <ReviewView />;
      case 'search':
        return <SearchView />;
      case 'draft':
        return <DraftView />;
      default:
        return <WelcomeEditor />;
    }
  };

  const breadcrumb =
    activeCase && activeTab
      ? `${activeCase.name}${activeTab.kind === 'document' && activeDoc ? ` › ${activeDoc.title}` : ` › ${activeTab.title}`}`
      : null;

  return (
    <div className="flex h-full min-w-0 flex-1 flex-col overflow-hidden">
      <EditorTabBar />
      {breadcrumb ? (
        <div
          className="flex items-center truncate"
          style={{
            height: 22,
            padding: '0 12px',
            fontSize: 11,
            color: 'var(--text-tertiary)',
            borderBottom: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-surface)',
          }}
        >
          {breadcrumb}
        </div>
      ) : null}
      <div className="relative min-h-0 flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          <motion.div
            key={activeTabId ?? activeView}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            className="h-full"
          >
            {renderView()}
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  );
};

export default EditorWorkbench;
