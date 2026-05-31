import type { DocxEditorRef } from '@eigenpal/docx-editor-react'

export type AIActionType = 'polish' | 'expand' | 'rewrite' | 'summarize'

export function getSelectedText(editor: DocxEditorRef): string | null {
  try {
    const info = editor.getSelectionInfo?.()
    return info?.selectedText ?? null
  } catch {
    return null
  }
}

export function buildPrompt(action: AIActionType, text: string): string {
  const prompts: Record<AIActionType, string> = {
    polish: `请润色以下文本，使其表达更专业流畅：\n\n${text}`,
    expand: `请扩写以下文本，增加更多技术细节：\n\n${text}`,
    rewrite: `请改写以下文本，保持原意但换一种表达方式：\n\n${text}`,
    summarize: `请总结以下文本的核心要点：\n\n${text}`,
  }
  return prompts[action]
}

export function getActionLabel(action: AIActionType): string {
  const labels: Record<AIActionType, string> = {
    polish: 'AI 润色',
    expand: 'AI 扩写',
    rewrite: 'AI 改写',
    summarize: 'AI 总结',
  }
  return labels[action]
}
