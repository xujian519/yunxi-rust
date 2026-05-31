import type { FC } from 'react';
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

const EditorWorkbench: FC = () => {
  const { activeView, editorTabs, activeTabId, activeCase, activeDocId, activeDocContent, docxMode, updateCaseDocument } = useApp();

  const activeTab = editorTabs.find((t) => t.id === activeTabId);
  const activeDoc = activeCase?.documents.find((d) => d.id === activeDocId);
  const showDrawings =
    activeTab?.kind === 'document' && activeDoc?.type === 'drawings';

  const renderView = () => {
    if (editorTabs.length === 0) return <WelcomeEditor />;
    if (showDrawings) return <DrawingsPlaceholder />;
    if (docxMode === 'docx') {
      const content = activeDoc?.contentMd || activeDocContent;
      return (
        <DocxEditorView
          markdownContent={content}
          mode="editing"
          onChange={(markdown) => {
            if (activeDocId) void updateCaseDocument(activeDocId, markdown);
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
