import React, { forwardRef, useEffect, useMemo, useRef, useState } from 'react'
import { DocxEditor, type DocxEditorRef, type EditorView } from '@eigenpal/docx-editor-react'
import type { Document } from '@eigenpal/docx-editor-core/types/document'
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
    const [error, setError] = useState<Error | null>(null)
    const editorRef = useRef<DocxEditorRef>(null)
    const [isEditorReady, setIsEditorReady] = useState(false)

    const document = useMemo(() => {
      if (initialDocument) {
        return initialDocument
      }
      if (markdownContent !== undefined) {
        return createDocumentWithText(markdownContent)
      }
      return createDocumentWithText('')
    }, [markdownContent, initialDocument])

    const handleDocumentChange = (updatedDocument: Document) => {
      if (!onChange) return

      try {
        const text = extractTextFromDocument(updatedDocument)
        onChange(text)
      } catch (err) {
        console.error('Failed to extract text from document:', err)
      }
    }

    const handleError = (err: Error) => {
      console.error('DocxEditor error:', err)
      setError(err)
      setIsLoading(false)
    }

    const handleEditorViewReady = (view: EditorView) => {
      setIsEditorReady(true)
      setIsLoading(false)

      if (onReady && editorRef.current) {
        onReady(editorRef.current)
      }

      if (typeof ref === 'function') {
        ref(editorRef.current)
      } else if (ref && 'current' in ref) {
        ref.current = editorRef.current
      }
    }

    useEffect(() => {
      return () => {
        setIsEditorReady(false)
      }
    }, [])

    if (error) {
      return (
        <div className="flex items-center justify-center h-full bg-red-50 text-red-600 p-8">
          <div className="text-center">
            <p className="text-lg font-semibold mb-2">编辑器加载失败</p>
            <p className="text-sm">{error.message}</p>
          </div>
        </div>
      )
    }

    if (isLoading) {
      return (
        <div className="flex items-center justify-center h-full bg-gray-50 text-gray-600">
          <p>加载编辑器…</p>
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
  }
)

DocxEditorView.displayName = 'DocxEditorView'

function extractTextFromDocument(document: Document): string {
  const paragraphs: string[] = []

  const body = document.package?.document
  if (!body?.content) {
    return ''
  }

  for (const element of body.content) {
    if (element.type === 'paragraph') {
      const paragraphText = extractTextFromParagraph(element)
      if (paragraphText.length > 0) {
        paragraphs.push(paragraphText)
      }
    } else if (element.type === 'table') {
      const tableText = extractTextFromTable(element)
      if (tableText.length > 0) {
        paragraphs.push(tableText)
      }
    }
  }

  return paragraphs.join('\n')
}

function extractTextFromRun(run: { type: string; content?: Array<{ type: string; text?: string }> }): string {
  if (!run.content) return ''
  return run.content
    .filter((c): c is { type: 'text'; text: string } => c.type === 'text' && !!c.text)
    .map((c) => c.text)
    .join('')
}

function extractTextFromParagraphContent(content: Array<{ type: string; content?: Array<{ type: string; text?: string }> }>): string {
  return content
    .filter((c) => c.type === 'run' && c.content)
    .map((c) => extractTextFromRun(c as { type: string; content: Array<{ type: string; text?: string }> }))
    .join('')
}

function extractTextFromTableCell(cell: { type: string; content?: Array<{ type: string; content?: Array<{ type: string; text?: string }> }> }): string {
  if (!cell.content) return ''
  return cell.content
    .filter((c) => c.type === 'paragraph')
    .map((p) => extractTextFromParagraphContent(p.content || []))
    .join(' | ')
}

function extractTextFromBlock(block: { type: string; content?: Array<{ type: string; content?: Array<{ type: string; text?: string }> }> }): string {
  if (block.type === 'paragraph') {
    return extractTextFromParagraphContent(block.content || [])
  }
  return ''
}

function extractTextFromTable(table: { type: string; rows?: Array<{ type: string; cells?: Array<{ type: string; content?: Array<{ type: string; content?: Array<{ type: string; text?: string }> }> }> }> }): string {
  if (!table.rows) return ''
  return table.rows
    .map((row) => {
      if (!row.cells) return ''
      return '| ' + row.cells
        .map((cell) => extractTextFromTableCell(cell))
        .join(' | ') + ' |'
    })
    .filter(Boolean)
    .join('\n')
}

function extractTextFromDocument(document: Document): string {
  const body = document.package?.document
  if (!body?.content) return ''

  return body.content
    .map((block: { type: string; content?: Array<{ type: string; content?: Array<{ type: string; text?: string }> }> } & { type: 'table'; rows?: Array<{ type: string; cells?: Array<{ type: string; content?: Array<{ type: string; content?: Array<{ type: string; text?: string }> }> }> }> }) => {
      if (block.type === 'paragraph') {
        return extractTextFromParagraphContent(block.content || [])
      }
      if (block.type === 'table') {
        return extractTextFromTable(block as Parameters<typeof extractTextFromTable>[0])
      }
      return ''
    })
    .filter(Boolean)
    .join('\n')
}