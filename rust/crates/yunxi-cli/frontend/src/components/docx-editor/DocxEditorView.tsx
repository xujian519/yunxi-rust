import { forwardRef, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { DocxEditor, type DocxEditorRef } from '@eigenpal/docx-editor-react'
import type { Document } from '@eigenpal/docx-editor-core/types/document'
import type { BlockContent } from '@eigenpal/docx-editor-core'
import { createDocumentWithText } from '@eigenpal/docx-editor-core'
import '@eigenpal/docx-editor-react/styles.css'

export interface DocxEditorViewProps {
  markdownContent?: string
  initialDocument?: Document
  mode?: 'editing' | 'suggesting' | 'viewing'
  onChange?: (markdown: string) => void
  onReady?: (ref: DocxEditorRef) => void
  readOnly?: boolean
  documentName?: string
  showToolbar?: boolean
}

export const DocxEditorView = forwardRef<DocxEditorRef, DocxEditorViewProps>(
  (
    {
      markdownContent,
      initialDocument,
      mode = 'editing',
      onChange,
      onReady,
      readOnly,
      documentName,
      showToolbar = true,
    },
    ref
  ) => {
    const [isLoading, setIsLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const editorRef = useRef<DocxEditorRef>(null)

    const document = useMemo(() => {
      if (initialDocument) return initialDocument
      return createDocumentWithText(markdownContent ?? '')
    }, [markdownContent, initialDocument])

    const handleDocumentChange = useCallback(
      (updatedDocument: Document) => {
        if (!onChange) return
        try {
          const text = extractText(updatedDocument)
          onChange(text)
        } catch (err) {
          console.error('Failed to extract text from document:', err)
        }
      },
      [onChange],
    )

    const handleError = useCallback((err: Error) => {
      console.error('DocxEditor error:', err)
      setError(err.message)
      setIsLoading(false)
    }, [])

    const handleEditorViewReady = useCallback(() => {
      setIsLoading(false)
      if (onReady && editorRef.current) onReady(editorRef.current)
      if (typeof ref === 'function') {
        ref(editorRef.current)
      } else if (ref && 'current' in ref) {
        ref.current = editorRef.current
      }
    }, [onReady, ref])

    useEffect(() => () => setIsLoading(true), [])

    if (error) {
      return (
        <div className="flex items-center justify-center h-full" style={{ backgroundColor: 'var(--bg-surface)', color: 'var(--status-error)' }}>
          <div className="text-center">
            <p className="text-sm font-semibold mb-1" style={{ color: 'var(--status-error)' }}>编辑器加载失败</p>
            <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>{error}</p>
          </div>
        </div>
      )
    }
    if (isLoading) {
      return (
        <div className="flex items-center justify-center h-full" style={{ backgroundColor: 'var(--bg-surface)', color: 'var(--text-secondary)' }}>
          <p className="text-sm">加载编辑器...</p>
        </div>
      )
    }

    return (
      <DocxEditor
        ref={editorRef}
        document={document}
        mode={mode}
        onChange={handleDocumentChange}
        onError={handleError}
        onEditorViewReady={handleEditorViewReady}
        readOnly={readOnly}
        showToolbar={showToolbar}
        showZoomControl={true}
        documentName={documentName}
        className="h-full w-full"
      />
    )
  },
)

DocxEditorView.displayName = 'DocxEditorView'

interface RunLike {
  content?: Array<{ type: string; text?: string }>
}

interface ParaLike {
  content?: RunLike[]
}

interface CellLike {
  content?: ParaLike[]
}

function extractText(body: { content?: BlockContent[] }): string {
  if (!body?.content) return ''
  return body.content
    .map((block) => {
      if (block.type === 'paragraph') return runsText(block.content ?? [])
      if (block.type === 'table') return tableText(block)
      return ''
    })
    .filter(Boolean)
    .join('\n')
}

function runsText(content: RunLike[]): string {
  return content
    .filter((c): c is { content: Array<{ type: string; text: string }> } =>
      'content' in c && !!c.content
    )
    .map((c) =>
      c.content
        .filter((x): x is { type: 'text'; text: string } => x.type === 'text' && !!x.text)
        .map((x) => x.text)
        .join(''),
    )
    .join('')
}

function tableText(table: { rows?: Array<{ cells?: CellLike[] }> }): string {
  if (!table.rows) return ''
  return table.rows
    .map((row) =>
      row.cells
        ? '| ' + row.cells.map((c) => runsText(c.content ?? [])).join(' | ') + ' |'
        : '',
    )
    .filter(Boolean)
    .join('\n')
}
