# DOCX 编辑器桌面端集成实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `@eigenpal/docx-editor-react` 集成到云熙智能体桌面端，实现 Markdown 与 DOCX 双模式编辑

**Architecture:** 新增 `DocxEditorView` 包装组件封装 docx-editor，通过 `useDocxConverter` hook 实现 Markdown ↔ DOCX 双向转换，利用 CSS 变量覆盖实现 UI 一致性，通过 DocumentAgent API 桥接 AI 编辑功能到右侧面板

**Tech Stack:** React 19, TypeScript, Tailwind CSS, shadcn/ui, Tauri 2, Vite, ProseMirror, `@eigenpal/docx-editor-react`

---

## 文件映射

### 新建文件

| 文件 | 行数 | 职责 |
|------|------|------|
| `src/components/docx-editor/DocxEditorView.tsx` | ~200 | docx-editor React 包装组件，处理初始化和事件 |
| `src/components/docx-editor/DocxEditorToolbar.tsx` | ~150 | 精简工具栏，隐藏默认标题栏，适配桌面端风格 |
| `src/components/docx-editor/docx-editor-theme-overrides.css` | ~100 | 覆盖 docx-editor 默认样式以匹配桌面端主题 |
| `src/components/docx-editor/index.ts` | ~20 | 导出 docx-editor 相关组件 |
| `src/components/workbench/ViewModeToggle.tsx` | ~80 | Markdown ↔ DOCX 模式切换按钮 |
| `src/hooks/useDocxConverter.ts` | ~120 | Markdown 与 DOCX 格式双向转换逻辑 |
| `src/utils/aiBridge.ts` | ~150 | 编辑器与右侧 AI 面板的通信桥接 |
| `src/components/docx-editor/__tests__/DocxEditorView.test.tsx` | ~80 | 组件基础渲染测试 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `package.json` | 添加 `@eigenpal/docx-editor-react` 及 ProseMirror peer dependencies |
| `vite.config.ts` | 确保 esbuild 处理 CommonJS 模块（docx-editor 依赖） |
| `src/context/AppProvider.tsx` | 添加 `docxMode` 状态、`setDocxMode` 方法、文档格式缓存 |
| `src/types/workspace.ts` | 添加 `DocxMode` 类型定义 |
| `src/components/workbench/EditorWorkbench.tsx` | 根据 `docxMode` 渲染 Markdown 或 DOCX 编辑器 |
| `src/components/workbench/EditorTabBar.tsx` | 添加视图模式切换按钮 |
| `src/components/center/DraftView.tsx` | 集成 `DocxEditorView`，支持双模式 |
| `src/components/center/ClaimsView.tsx` | 集成 `DocxEditorView`，支持双模式 |
| `src/components/center/ReviewView.tsx` | 集成 `DocxEditorView`，支持 tracked changes |

---

## Phase 1: 基础集成与主题适配（2-3 天）

### Task 1: 安装 docx-editor 依赖

**Files:**
- Modify: `package.json`

**Context:** 需要安装新版 `@eigenpal/docx-editor-react`（注意：用户下载的 0.x 版本已废弃，新版包名为 `@eigenpal/docx-editor-react`）。同时需要安装 ProseMirror 的 peer dependencies。

**注意:** 实际安装前请确认 npm registry 上最新版本号。当前设计假设安装 `^1.0.0`。

- [ ] **Step 1: 添加依赖到 package.json**

```bash
# 先确认可用版本
npm view @eigenpal/docx-editor-react versions --json | tail -5
```

在 `package.json` 的 `dependencies` 中添加：
```json
    "@eigenpal/docx-editor-react": "^1.0.0",
    "prosemirror-commands": "^1.5.2",
    "prosemirror-dropcursor": "^1.8.2",
    "prosemirror-history": "^1.4.0",
    "prosemirror-keymap": "^1.2.2",
    "prosemirror-model": "^1.19.4",
    "prosemirror-state": "^1.4.3",
    "prosemirror-tables": "^1.8.5",
    "prosemirror-transform": "^1.10.2",
    "prosemirror-view": "^1.41.6"
```

- [ ] **Step 2: 安装依赖**

```bash
cd /Users/xujian/projects/YunXi/rust/crates/yunxi-cli/frontend
npm install
```

Expected: 安装成功，无 peer dependency 警告。

- [ ] **Step 3: 验证安装**

```bash
ls node_modules/@eigenpal/docx-editor-react
ls node_modules/prosemirror-view
```

Expected: 两个目录都存在。

- [ ] **Step 4: Commit**

```bash
git add package.json package-lock.json
git commit -m "deps(frontend): 添加 docx-editor-react 及 ProseMirror 依赖"
```

---

### Task 2: 创建主题覆盖 CSS

**Files:**
- Create: `src/components/docx-editor/docx-editor-theme-overrides.css`

**Context:** docx-editor 的样式被隔离在 `.ep-root` 下，但仍有硬编码颜色。通过 CSS 覆盖将其适配到云熙桌面端的暖色调主题。

- [ ] **Step 1: 创建主题覆盖文件**

创建 `src/components/docx-editor/docx-editor-theme-overrides.css`：

```css
/**
 * DOCX Editor 主题覆盖
 * 
 * 将 docx-editor 的默认样式适配到云熙桌面端主题。
 * 所有规则限定在 .ep-root 下，避免污染全局样式。
 */

/* === 工具栏和 Chrome === */
.ep-root .bg-white {
  background-color: var(--bg-elevated, #FFFFFF) !important;
}

.ep-root .bg-slate-900 {
  background-color: var(--bg-base, #1C1A18) !important;
}

/* 覆盖 Google 蓝色为桌面端强调色 */
.ep-root {
  --doc-primary: var(--accent-primary, #4A7C6F);
  --doc-primary-hover: var(--accent-primary-hover, #3D6A5E);
  --doc-primary-light: var(--accent-primary-muted, rgba(74, 124, 111, 0.12));
}

/* 按钮 hover 效果 */
.ep-root button:hover {
  background-color: var(--bg-sidebar-active, rgba(230, 226, 220, 0.85));
}

/* === 对话框/下拉菜单 === */
.ep-root [role="dialog"],
.ep-root [data-radix-popper-content-wrapper] {
  background-color: var(--bg-elevated, #FFFFFF);
  border-color: var(--border-primary, rgba(0, 0, 0, 0.06));
  color: var(--text-primary, #1A1814);
}

/* Tooltip 背景 */
.ep-root .fixed.z-50.px-2.py-1 {
  background-color: var(--bg-base, #1C1A18);
  color: var(--text-inverse, #FFFFFF);
}

/* === 选择高亮 === */
.ep-root .docx-run-editable::selection,
.ep-root .docx-run-editable *::selection,
.ep-root .docx-run::selection,
.ep-root .docx-run *::selection,
.ep-root [contenteditable='true']::selection,
.ep-root [contenteditable='true'] *::selection {
  background-color: var(--accent-primary-muted, rgba(74, 124, 111, 0.3)) !important;
}

/* === 暗色模式适配 === */
.dark .ep-root {
  --doc-bg: var(--bg-base, #1C1A18);
  --doc-text: var(--text-primary, #F0EDE8);
  --doc-text-muted: var(--text-secondary, #9A9590);
  --doc-border: var(--border-primary, rgba(255, 255, 255, 0.08));
  --doc-bg-hover: var(--bg-sidebar-active, rgba(60, 58, 56, 0.85));
}

.dark .ep-root .bg-white {
  background-color: var(--bg-elevated, #2C2A28) !important;
}

/* === 编辑器页面保持白色（WYSIWYG） === */
.ep-root .docx-editor-page {
  background-color: #FFFFFF;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12), 0 1px 2px rgba(0, 0, 0, 0.08);
}

.dark .ep-root .docx-editor-page {
  background-color: #FFFFFF; /* 页面在暗色模式下仍保持白色 */
}
```

- [ ] **Step 2: 在入口文件中导入 CSS**

修改 `src/main.tsx`，在现有 import 后添加：

```typescript
import './components/docx-editor/docx-editor-theme-overrides.css'
```

**注意:** 必须在 docx-editor 的样式之后导入，以确保覆盖生效。如果 docx-editor 的样式是动态导入的，需要调整加载顺序。

- [ ] **Step 3: Commit**

```bash
git add src/components/docx-editor/docx-editor-theme-overrides.css src/main.tsx
git commit -m "style(docx-editor): 添加主题覆盖 CSS 适配桌面端风格"
```

---

### Task 3: 创建 DocxEditorView 基础组件

**Files:**
- Create: `src/components/docx-editor/DocxEditorView.tsx`
- Create: `src/components/docx-editor/index.ts`

**Context:** 这是 docx-editor 的核心包装组件。由于需要 DOM，在 SSR 环境下需要使用 dynamic import，但桌面端是客户端渲染，可以直接导入。

**注意:** 新版 API 可能有变化，以下代码基于 0.x 版本 API 推测，实际使用时需要根据新版文档调整。

- [ ] **Step 1: 创建 DocxEditorView 组件**

创建 `src/components/docx-editor/DocxEditorView.tsx`：

```typescript
import { useRef, useCallback, useEffect, useState } from 'react'
import type { FC } from 'react'
import { DocxEditor, type DocxEditorRef } from '@eigenpal/docx-editor-react'
import '@eigenpal/docx-editor-react/styles.css'
import type { Document } from '@eigenpal/docx-editor-react/core/types/document'

export interface DocxEditorViewProps {
  /** 初始文档内容（docx Document 对象或 ArrayBuffer） */
  initialDocument?: Document | ArrayBuffer
  /** 是否启用修订追踪模式 */
  trackChanges?: boolean
  /** 文档变更回调 */
  onChange?: (document: Document) => void
  /** 编辑器准备好后的回调 */
  onReady?: (ref: DocxEditorRef) => void
  /** 是否只读 */
  readOnly?: boolean
}

const DocxEditorView: FC<DocxEditorViewProps> = ({
  initialDocument,
  trackChanges = false,
  onChange,
  onReady,
  readOnly = false,
}) => {
  const editorRef = useRef<DocxEditorRef>(null)
  const [isReady, setIsReady] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleReady = useCallback(() => {
    setIsReady(true)
    if (editorRef.current && onReady) {
      onReady(editorRef.current)
    }
  }, [onReady])

  const handleChange = useCallback(
    (doc: Document) => {
      if (onChange) {
        onChange(doc)
      }
    },
    [onChange],
  )

  const handleError = useCallback((err: Error) => {
    setError(err.message)
    console.error('DocxEditor error:', err)
  }, [])

  useEffect(() => {
    // 清理函数
    return () => {
      // docx-editor 可能需要在卸载时执行清理
    }
  }, [])

  if (error) {
    return (
      <div
        className="flex h-full flex-col items-center justify-center"
        style={{ color: 'var(--status-error)' }}
      >
        <p className="text-sm">编辑器加载失败</p>
        <p className="mt-2 text-xs opacity-70">{error}</p>
      </div>
    )
  }

  return (
    <div className="ep-root flex h-full flex-col">
      <DocxEditor
        ref={editorRef}
        document={initialDocument}
        mode={readOnly ? 'reading' : trackChanges ? 'review' : 'editing'}
        onChange={handleChange}
        onReady={handleReady}
        onError={handleError}
        className="flex-1"
      />
      {!isReady && (
        <div
          className="absolute inset-0 flex items-center justify-center"
          style={{ backgroundColor: 'var(--bg-surface)' }}
        >
          <div className="text-sm" style={{ color: 'var(--text-secondary)' }}>
            加载编辑器…
          </div>
        </div>
      )}
    </div>
  )
}

export default DocxEditorView
```

- [ ] **Step 2: 创建索引文件**

创建 `src/components/docx-editor/index.ts`：

```typescript
export { default as DocxEditorView } from './DocxEditorView'
export type { DocxEditorViewProps } from './DocxEditorView'
```

- [ ] **Step 3: 验证类型检查**

```bash
cd /Users/xujian/projects/YunXi/rust/crates/yunxi-cli/frontend
npx tsc --noEmit
```

Expected: 无类型错误。如果有 API 不匹配，根据错误信息调整。

- [ ] **Step 4: Commit**

```bash
git add src/components/docx-editor/
git commit -m "feat(docx-editor): 创建 DocxEditorView 基础包装组件"
```

---

### Task 4: 在 AppProvider 中增加 docxMode 状态

**Files:**
- Modify: `src/types/workspace.ts`
- Modify: `src/context/AppProvider.tsx`

**Context:** 需要在全局状态中增加当前文档的编辑模式（Markdown 或 DOCX）。模式应该按文档（或按视图）持久化，而不是全局的。

- [ ] **Step 1: 添加 DocxMode 类型**

修改 `src/types/workspace.ts`，在文件末尾添加：

```typescript
/**
 * 文档编辑模式
 */
export type DocxMode = 'markdown' | 'docx'

/**
 * 文档编辑模式存储（按文档 ID）
 */
export type DocxModeMap = Record<string, DocxMode>
```

- [ ] **Step 2: 读取当前文件确认接口**

```bash
grep -n "export interface AppContextValue" src/context/AppProvider.tsx
```

Expected: 找到接口定义行号。

- [ ] **Step 3: 在 AppContextValue 接口中添加 docxMode 相关字段**

在 `AppContextValue` 接口中（约第 84 行之后）添加：

```typescript
  /** 当前活动文档的编辑模式 */
  docxMode: DocxMode
  /** 设置当前活动文档的编辑模式 */
  setDocxMode: (mode: DocxMode) => void
  /** 获取指定文档的编辑模式 */
  getDocxMode: (docId: string) => DocxMode
```

- [ ] **Step 4: 在 AppProvider 组件中添加状态逻辑**

在 `AppProvider` 组件内部（约第 400 行左右，其他 useState 定义附近）添加：

```typescript
  const [docxModes, setDocxModes] = useState<DocxModeMap>(() => {
    try {
      const raw = localStorage.getItem('yunxi-docx-modes')
      return raw ? JSON.parse(raw) : {}
    } catch {
      return {}
    }
  })
```

然后在回调定义区域添加：

```typescript
  const docxMode = useMemo<DocxMode>(() => {
    if (!activeDocId) return 'markdown'
    return docxModes[activeDocId] || 'markdown'
  }, [docxModes, activeDocId])

  const setDocxMode = useCallback(
    (mode: DocxMode) => {
      if (!activeDocId) return
      setDocxModes((prev) => {
        const next = { ...prev, [activeDocId]: mode }
        localStorage.setItem('yunxi-docx-modes', JSON.stringify(next))
        return next
      })
    },
    [activeDocId],
  )

  const getDocxMode = useCallback(
    (docId: string) => docxModes[docId] || 'markdown',
    [docxModes],
  )
```

- [ ] **Step 5: 在 value useMemo 中添加新字段**

在 `value` useMemo 对象（约第 1530 行）中添加：

```typescript
      docxMode,
      setDocxMode,
      getDocxMode,
```

- [ ] **Step 6: 验证类型检查**

```bash
npx tsc --noEmit
```

Expected: 无类型错误。

- [ ] **Step 7: Commit**

```bash
git add src/types/workspace.ts src/context/AppProvider.tsx
git commit -m "feat(context): 在 AppProvider 中添加 docxMode 状态管理"
```

---

### Task 5: 创建 ViewModeToggle 组件

**Files:**
- Create: `src/components/workbench/ViewModeToggle.tsx`

**Context:** 在编辑器标签栏或工具栏中提供 Markdown ↔ DOCX 模式切换按钮。

- [ ] **Step 1: 创建切换按钮组件**

创建 `src/components/workbench/ViewModeToggle.tsx`：

```typescript
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
```

- [ ] **Step 2: Commit**

```bash
git add src/components/workbench/ViewModeToggle.tsx
git commit -m "feat(ui): 创建 Markdown/DOCX 模式切换按钮"
```

---

### Task 6: 修改 EditorTabBar 添加模式切换

**Files:**
- Modify: `src/components/workbench/EditorTabBar.tsx`

**Context:** 在标签栏右侧添加 ViewModeToggle，让用户可以切换当前文档的编辑模式。

- [ ] **Step 1: 导入 ViewModeToggle**

在 `EditorTabBar.tsx` 的 import 区域添加：

```typescript
import ViewModeToggle from './ViewModeToggle'
```

- [ ] **Step 2: 在渲染中添加切换按钮**

在 `EditorTabBar` 组件的 return 语句中，在 tabs 列表后添加：

```typescript
      {editorTabs.length > 0 && (
        <div className="flex flex-1 items-center justify-end px-2">
          <ViewModeToggle />
        </div>
      )}
```

完整修改后的 return 结构：

```typescript
  return (
    <div
      className="flex items-stretch overflow-x-auto custom-scrollbar"
      style={{
        height: 35,
        minHeight: 35,
        backgroundColor: 'var(--bg-elevated)',
        borderBottom: '1px solid var(--border-primary)',
      }}
    >
      {editorTabs.map((tab, index) => {
        // ... 现有 tab 渲染代码不变 ...
      })}
      {editorTabs.length > 0 && (
        <div className="flex flex-1 items-center justify-end px-2">
          <ViewModeToggle />
        </div>
      )}
    </div>
  )
```

- [ ] **Step 3: 验证渲染**

```bash
npm run dev
```

在浏览器中打开桌面端，确认：
- [ ] 标签栏右侧出现 Markdown/DOCX 切换按钮
- [ ] 点击切换按钮可以切换模式（虽然 DOCX 编辑器还未接入）
- [ ] 模式切换后 localStorage 中 `yunxi-docx-modes` 被更新

- [ ] **Step 4: Commit**

```bash
git add src/components/workbench/EditorTabBar.tsx
git commit -m "feat(ui): 在 EditorTabBar 中添加模式切换按钮"
```

---

### Task 7: 修改 EditorWorkbench 支持模式切换

**Files:**
- Modify: `src/components/workbench/EditorWorkbench.tsx`

**Context:** 根据当前 `docxMode` 决定渲染 Markdown 编辑器还是 DOCX 编辑器。目前先用占位符测试 DOCX 模式。

- [ ] **Step 1: 导入 DocxEditorView**

在 `EditorWorkbench.tsx` 的 import 区域添加：

```typescript
import { DocxEditorView } from '@/components/docx-editor'
```

- [ ] **Step 2: 修改 renderView 函数**

修改 `renderView` 函数，根据 `docxMode` 渲染不同内容：

```typescript
  const renderView = () => {
    if (editorTabs.length === 0) return <WelcomeEditor />
    if (showDrawings) return <DrawingsPlaceholder />
    
    // DOCX 模式渲染
    if (docxMode === 'docx') {
      return (
        <DocxEditorView
          onChange={(doc) => {
            console.log('Document changed:', doc)
          }}
        />
      )
    }
    
    // Markdown 模式渲染（现有逻辑）
    switch (activeView) {
      case 'claims':
        return <ClaimsView />
      case 'compare':
        return <CompareView />
      case 'review':
        return <ReviewView />
      case 'search':
        return <SearchView />
      case 'draft':
        return <DraftView />
      default:
        return <WelcomeEditor />
    }
  }
```

同时需要从 `useApp()` 中解构出 `docxMode`：

```typescript
  const { activeView, editorTabs, activeTabId, activeCase, activeDocId, docxMode } = useApp()
```

- [ ] **Step 3: 验证切换**

```bash
npm run dev
```

测试步骤：
1. 打开一个案件文档
2. 点击标签栏的 "DOCX" 按钮
3. 确认中央区域渲染了 docx-editor（或错误提示）
4. 切换回 "Markdown" 确认正常显示

- [ ] **Step 4: Commit**

```bash
git add src/components/workbench/EditorWorkbench.tsx
git commit -m "feat(workbench): EditorWorkbench 支持 DOCX 模式切换"
```

---

### Task 8: Phase 1 验收与决策检查点

**目标**: 验证 UI 一致性是否达到可接受水平

- [ ] **Step 1: 检查样式覆盖效果**

启动桌面端，切换到 DOCX 模式，检查以下元素：
- [ ] 工具栏背景色是否与桌面端 `--bg-elevated` 一致
- [ ] 按钮 hover 效果是否与桌面端一致
- [ ] 编辑器页面是否为白色（正确）
- [ ] 无样式泄漏（桌面端其他组件未被影响）

- [ ] **Step 2: 检查暗色模式**

切换到暗色主题，检查：
- [ ] 编辑器 chrome（工具栏、对话框）是否适配暗色
- [ ] 编辑器页面是否仍为白色（正确）

- [ ] **Step 3: 决策**

如果以上检查全部通过，继续 Phase 2。

如果有超过 3 个关键 UI 元素无法通过简单 CSS 覆盖适配，执行回退：
1. 保留当前代码作为基础
2. 修改 `DocxEditorView` 为仅支持导入/导出的轻量组件
3. 移除 ProseMirror 依赖
4. 更新设计文档说明回退原因

---

## Phase 2: DraftView DOCX 模式试点（3-4 天）

### Task 9: 创建 useDocxConverter hook

**Files:**
- Create: `src/hooks/useDocxConverter.ts`

**Context:** 实现 Markdown 与 DOCX Document 对象之间的双向转换。这是连接现有 Markdown 存储和 docx-editor 的桥梁。

- [ ] **Step 1: 创建转换 hook**

创建 `src/hooks/useDocxConverter.ts`：

```typescript
import { useCallback, useRef } from 'react'
import { createDocumentWithText, parseDocx } from '@eigenpal/docx-editor-react/core'
import { serializeDocx } from '@eigenpal/docx-editor-react/core/serializer'
import type { Document } from '@eigenpal/docx-editor-react/core/types/document'

export interface UseDocxConverterReturn {
  /** Markdown 转 DOCX Document */
  mdToDocx: (markdown: string) => Document
  /** DOCX Document 转 Markdown（纯文本提取） */
  docxToMd: (doc: Document) => string
  /** ArrayBuffer（.docx 文件）转 Document */
  bufferToDocx: (buffer: ArrayBuffer) => Promise<Document>
  /** Document 转 ArrayBuffer（用于导出） */
  docxToBuffer: (doc: Document) => Promise<ArrayBuffer>
}

/**
 * 简单的 Markdown 解析器
 * 将基础 Markdown 转为 docx-editor 可识别的格式
 */
function parseMarkdownToDocx(markdown: string): Document {
  // 使用 docx-editor 的 API 创建带格式的文档
  // 这里先用纯文本方式创建，后续可以扩展支持标题、列表等
  return createDocumentWithText(markdown)
}

/**
 * 从 DOCX Document 提取纯文本
 * 这是简化版本，后续可以保留更多格式信息
 */
function extractTextFromDocx(doc: Document): string {
  try {
    // 尝试使用 serializeDocx 获取文本内容
    const serialized = serializeDocx(doc)
    // 如果返回的是字符串，直接使用
    if (typeof serialized === 'string') {
      return serialized
    }
    // 否则尝试提取 body 中的文本
    if (doc.body && doc.body.content) {
      return doc.body.content
        .map((block) => {
          if ('paragraph' in block && block.paragraph) {
            return block.paragraph.runs
              .map((run) => run.text || '')
              .join('')
          }
          return ''
        })
        .join('\n')
    }
    return ''
  } catch (e) {
    console.error('Failed to extract text from docx:', e)
    return ''
  }
}

export function useDocxConverter(): UseDocxConverterReturn {
  const converterRef = useRef<{
    mdToDocx: (md: string) => Document
    docxToMd: (doc: Document) => string
  }>({
    mdToDocx: parseMarkdownToDocx,
    docxToMd: extractTextFromDocx,
  })

  const mdToDocx = useCallback((markdown: string): Document => {
    return converterRef.current.mdToDocx(markdown)
  }, [])

  const docxToMd = useCallback((doc: Document): string => {
    return converterRef.current.docxToMd(doc)
  }, [])

  const bufferToDocx = useCallback(async (buffer: ArrayBuffer): Promise<Document> => {
    return parseDocx(buffer)
  }, [])

  const docxToBuffer = useCallback(async (doc: Document): Promise<ArrayBuffer> => {
    // 使用 docx-core 的序列化功能
    // 注意：实际 API 可能不同，需要根据文档调整
    const result = serializeDocx(doc)
    if (result instanceof ArrayBuffer) {
      return result
    }
    if (typeof result === 'string') {
      return new TextEncoder().encode(result).buffer
    }
    throw new Error('Unsupported serialize result type')
  }, [])

  return { mdToDocx, docxToMd, bufferToDocx, docxToBuffer }
}
```

- [ ] **Step 2: 验证类型检查**

```bash
npx tsc --noEmit
```

Expected: 无类型错误。如果有 API 不匹配，根据 `@eigenpal/docx-editor-react` 的实际导出调整 import 路径。

- [ ] **Step 3: Commit**

```bash
git add src/hooks/useDocxConverter.ts
git commit -m "feat(hooks): 创建 useDocxConverter Markdown↔DOCX 转换 hook"
```

---

### Task 10: 修改 DocxEditorView 支持初始文档加载

**Files:**
- Modify: `src/components/docx-editor/DocxEditorView.tsx`

**Context:** 让 DocxEditorView 能够接收 Markdown 内容，自动转换为 DOCX Document 后渲染。

- [ ] **Step 1: 更新组件接口**

修改 `DocxEditorViewProps` 接口，添加 `markdownContent` 属性：

```typescript
export interface DocxEditorViewProps {
  /** Markdown 格式初始内容（与 initialDocument 二选一） */
  markdownContent?: string
  /** DOCX 格式初始内容（与 markdownContent 二选一） */
  initialDocument?: Document | ArrayBuffer
  /** 是否启用修订追踪模式 */
  trackChanges?: boolean
  /** 文档变更回调（返回 Markdown） */
  onChange?: (markdown: string) => void
  /** 编辑器准备好后的回调 */
  onReady?: (ref: DocxEditorRef) => void
  /** 是否只读 */
  readOnly?: boolean
}
```

- [ ] **Step 2: 在组件内部使用转换 hook**

修改组件逻辑，在初始化时将 Markdown 转为 DOCX：

```typescript
import { useDocxConverter } from '@/hooks/useDocxConverter'

const DocxEditorView: FC<DocxEditorViewProps> = ({
  markdownContent,
  initialDocument,
  trackChanges = false,
  onChange,
  onReady,
  readOnly = false,
}) => {
  const editorRef = useRef<DocxEditorRef>(null)
  const [isReady, setIsReady] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { mdToDocx, docxToMd } = useDocxConverter()

  // 将初始内容转为 Document
  const document = useMemo(() => {
    if (initialDocument) {
      if (initialDocument instanceof ArrayBuffer) {
        // 异步加载 ArrayBuffer，这里先用同步方式处理
        // 实际应该使用 useEffect + async
        return undefined
      }
      return initialDocument
    }
    if (markdownContent) {
      try {
        return mdToDocx(markdownContent)
      } catch (e) {
        setError(e instanceof Error ? e.message : '转换失败')
        return undefined
      }
    }
    return undefined
  }, [initialDocument, markdownContent, mdToDocx])

  const handleChange = useCallback(
    (doc: Document) => {
      if (onChange) {
        try {
          const markdown = docxToMd(doc)
          onChange(markdown)
        } catch (e) {
          console.error('Failed to convert docx to markdown:', e)
        }
      }
    },
    [onChange, docxToMd],
  )

  // ... 其余逻辑不变
}
```

- [ ] **Step 3: 验证类型检查**

```bash
npx tsc --noEmit
```

- [ ] **Step 4: Commit**

```bash
git add src/components/docx-editor/DocxEditorView.tsx
git commit -m "feat(docx-editor): DocxEditorView 支持 Markdown 初始内容"
```

---

### Task 11: 修改 EditorWorkbench 传递文档内容

**Files:**
- Modify: `src/components/workbench/EditorWorkbench.tsx`

**Context:** 在 DOCX 模式下，需要将当前文档的 Markdown 内容传递给 DocxEditorView，并处理编辑器返回的变更。

- [ ] **Step 1: 修改 DOCX 模式渲染逻辑**

将之前的占位符替换为完整实现：

```typescript
    // DOCX 模式渲染
    if (docxMode === 'docx') {
      const activeDoc = activeCase?.documents.find((d) => d.id === activeDocId)
      const content = activeDoc?.contentMd || ''
      
      return (
        <DocxEditorView
          markdownContent={content}
          onChange={(markdown) => {
            if (activeDocId) {
              void updateCaseDocument(activeDocId, markdown)
            }
          }}
        />
      )
    }
```

需要从 `useApp()` 中解构 `updateCaseDocument`：

```typescript
  const { activeView, editorTabs, activeTabId, activeCase, activeDocId, docxMode, updateCaseDocument } = useApp()
```

- [ ] **Step 2: 测试文档打开与编辑**

```bash
npm run dev
```

测试步骤：
1. 打开一个说明书案件
2. 切换到 DOCX 模式
3. 在编辑器中输入内容
4. 等待 500ms（debounce）
5. 切换回 Markdown 模式
6. 确认 Markdown 内容已更新

- [ ] **Step 3: Commit**

```bash
git add src/components/workbench/EditorWorkbench.tsx
git commit -m "feat(workbench): EditorWorkbench 传递文档内容到 DOCX 编辑器"
```

---

### Task 12: Phase 2 验收

- [ ] **Step 1: 功能验证**

- [ ] 说明书可在 Markdown 和 DOCX 模式间切换
- [ ] DOCX 模式编辑后，Markdown 内容正确同步
- [ ] 切换模式不丢失内容
- [ ] 性能可接受（大文档 >100 页不卡顿）

- [ ] **Step 2: UI 验证**

- [ ] 编辑器在桌面端正常渲染
- [ ] 工具栏颜色与桌面端主题一致
- [ ] 无样式污染

---

## Phase 3: 多视图扩展与修订追踪（4-5 天）

### Task 13: 扩展 ClaimsView 支持 DOCX 模式

**Files:**
- Modify: `src/components/center/ClaimsView.tsx`

**Context:** 权利要求书视图需要支持双模式。权利要求书有特殊的行号显示和语法高亮，在 DOCX 模式下可以保留这些功能作为编辑器装饰。

- [ ] **Step 1: 重构 ClaimsView 支持模式切换**

在 `ClaimsView` 中根据 `docxMode` 渲染不同内容：

```typescript
import { useApp } from '@/context/AppProvider'
import { DocxEditorView } from '@/components/docx-editor'

const ClaimsView: FC = () => {
  const { activeDocContent, docxMode, activeDocId, updateCaseDocument } = useApp()
  // ... 现有状态

  if (docxMode === 'docx') {
    return (
      <DocxEditorView
        markdownContent={activeDocContent}
        onChange={(markdown) => {
          if (activeDocId) {
            void updateCaseDocument(activeDocId, markdown)
          }
        }}
      />
    )
  }

  // 现有 Markdown 渲染逻辑不变
  // ...
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/center/ClaimsView.tsx
git commit -m "feat(claims): ClaimsView 支持 DOCX 模式"
```

---

### Task 14: 扩展 ReviewView 支持修订追踪

**Files:**
- Modify: `src/components/center/ReviewView.tsx`

**Context:** 审查意见答复是最需要修订追踪功能的场景。用户可以在 DOCX 模式下开启 tracked changes，所有修改都会标记。

- [ ] **Step 1: 重构 ReviewView 支持双模式和修订追踪**

```typescript
import { useApp } from '@/context/AppProvider'
import { DocxEditorView } from '@/components/docx-editor'
import { useState } from 'react'

const ReviewView: FC = () => {
  const { getDocumentByType, updateCaseDocument, activeCase, docxMode } = useApp()
  const reviewDoc = getDocumentByType('review')
  const [trackChanges, setTrackChanges] = useState(true)

  if (docxMode === 'docx') {
    return (
      <div className="flex h-full flex-col">
        <div
          className="flex items-center gap-2 px-3"
          style={{
            height: 32,
            borderBottom: '1px solid var(--border-primary)',
            backgroundColor: 'var(--bg-elevated)',
          }}
        >
          <label className="flex items-center gap-1 text-xs" style={{ color: 'var(--text-secondary)' }}>
            <input
              type="checkbox"
              checked={trackChanges}
              onChange={(e) => setTrackChanges(e.target.checked)}
              className="rounded"
            />
            修订模式
          </label>
        </div>
        <div className="flex-1">
          <DocxEditorView
            markdownContent={reviewDoc?.contentMd || ''}
            trackChanges={trackChanges}
            onChange={(markdown) => {
              if (reviewDoc?.id) {
                void updateCaseDocument(reviewDoc.id, markdown)
              }
            }}
          />
        </div>
      </div>
    )
  }

  // 现有 Markdown 渲染逻辑不变
  // ...
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/center/ReviewView.tsx
git commit -m "feat(review): ReviewView 支持 DOCX 模式和修订追踪"
```

---

### Task 15: 扩展 CompareView 和 SearchView

**Files:**
- Modify: `src/components/center/CompareView.tsx`
- Modify: `src/components/center/SearchView.tsx`

**Context:** 对比分析和检索视图相对简单，主要是展示内容，可以只读模式渲染 DOCX。

- [ ] **Step 1: 修改 CompareView**

在 `CompareView` 中添加 DOCX 模式支持：

```typescript
  if (docxMode === 'docx') {
    // 对比视图在 DOCX 模式下可以显示第一个文档
    const claimsDoc = getDocumentByType('claims')
    return (
      <DocxEditorView
        markdownContent={claimsDoc?.contentMd || ''}
        readOnly
      />
    )
  }
```

- [ ] **Step 2: 修改 SearchView**

在 `SearchView` 中添加 DOCX 模式支持（类似 CompareView）。

- [ ] **Step 3: Commit**

```bash
git add src/components/center/CompareView.tsx src/components/center/SearchView.tsx
git commit -m "feat(views): CompareView 和 SearchView 支持 DOCX 只读模式"
```

---

### Task 16: Phase 3 验收

- [ ] **Step 1: 功能验证**

- [ ] 全部视图支持 DOCX 模式切换
- [ ] 审查意见答复可开启修订模式
- [ ] 修订标记正确显示
- [ ] 接受/拒绝修订后内容正确更新

---

## Phase 4: AI 编辑桥接（3-4 天）

### Task 17: 创建 aiBridge 模块

**Files:**
- Create: `src/utils/aiBridge.ts`

**Context:** 实现编辑器内选中文字与右侧 AI 面板的通信。用户在 DOCX 编辑器中选中文字后，可以触发 AI 润色/扩写/改写，结果通过 DocumentAgent API 应用到编辑器。

- [ ] **Step 1: 创建桥接模块**

创建 `src/utils/aiBridge.ts`：

```typescript
import type { Document } from '@eigenpal/docx-editor-react/core/types/document'
import type { DocxEditorRef } from '@eigenpal/docx-editor-react'

export type AIAction = 'polish' | 'expand' | 'rewrite' | 'summarize'

export interface AIEditRequest {
  action: AIAction
  selectedText: string
  context: string
  documentId: string
}

export interface AIEditResponse {
  replacement: string
  explanation?: string
}

/**
 * 从编辑器获取选中内容
 */
export function getSelectedText(editorRef: DocxEditorRef): string | null {
  try {
    // 使用 DocumentAgent API 获取选中内容
    // 实际 API 可能不同，需要根据文档调整
    const selection = editorRef.getSelection?.()
    if (selection && selection.text) {
      return selection.text
    }
    return null
  } catch (e) {
    console.error('Failed to get selection:', e)
    return null
  }
}

/**
 * 构建 AI 编辑提示词
 */
export function buildAIPrompt(action: AIAction, text: string, context?: string): string {
  const prompts: Record<AIAction, string> = {
    polish: `请润色以下文本，使其表达更专业、流畅：\n\n${text}`,
    expand: `请扩展以下文本，增加更多细节和说明：\n\n${text}`,
    rewrite: `请改写以下文本，保持原意但使用不同的表达方式：\n\n${text}`,
    summarize: `请总结以下文本的核心要点：\n\n${text}`,
  }

  let prompt = prompts[action]
  if (context) {
    prompt = `上下文：${context}\n\n${prompt}`
  }
  return prompt
}

/**
 * 应用 AI 编辑结果到编辑器
 */
export function applyAIEdit(
  editorRef: DocxEditorRef,
  originalText: string,
  replacement: string,
  options?: { trackChanges?: boolean },
): boolean {
  try {
    // 使用 DocumentAgent API 替换文本
    // 实际 API 可能不同
    if (options?.trackChanges) {
      // 在修订模式下应用修改
      editorRef.replaceText?.(originalText, replacement, { trackChanges: true })
    } else {
      editorRef.replaceText?.(originalText, replacement)
    }
    return true
  } catch (e) {
    console.error('Failed to apply AI edit:', e)
    return false
  }
}

/**
 * 发送 AI 编辑请求（集成到右侧 AI 面板）
 */
export async function sendAIEditRequest(
  request: AIEditRequest,
  sendMessage: (content: string) => Promise<void>,
): Promise<void> {
  const prompt = buildAIPrompt(request.action, request.selectedText, request.context)
  await sendMessage(prompt)
}
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/aiBridge.ts
git commit -m "feat(ai): 创建 AI 编辑桥接模块"
```

---

### Task 18: 在 DocxEditorView 中集成右键 AI 菜单

**Files:**
- Modify: `src/components/docx-editor/DocxEditorView.tsx`

**Context:** 在编辑器中增加右键菜单选项，让用户可以选中文字后调用 AI 功能。

- [ ] **Step 1: 添加右键菜单支持**

修改 `DocxEditorView`，增加 `onAIAction` 回调：

```typescript
export interface DocxEditorViewProps {
  // ... 现有属性
  /** AI 编辑动作回调 */
  onAIAction?: (action: AIAction, selectedText: string) => void
}
```

在组件内部处理右键菜单或快捷键：

```typescript
  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      // 如果有选中文本且 onAIAction 存在，显示自定义右键菜单
      // 或者使用 docx-editor 内置的上下文菜单扩展
      if (onAIAction && editorRef.current) {
        const selectedText = getSelectedText(editorRef.current)
        if (selectedText) {
          // 显示 AI 操作菜单
          // 实际实现需要使用 docx-editor 的上下文菜单 API
        }
      }
    },
    [onAIAction],
  )
```

- [ ] **Step 2: Commit**

```bash
git add src/components/docx-editor/DocxEditorView.tsx
git commit -m "feat(docx-editor): 集成 AI 右键菜单支持"
```

---

### Task 19: 在 AppProvider 中集成 AI 编辑发送

**Files:**
- Modify: `src/context/AppProvider.tsx`

**Context:** 将 AI 编辑请求发送到右侧 AI 面板。需要复用现有的 `send` 方法。

- [ ] **Step 1: 添加 sendAIEdit 方法**

在 `AppContextValue` 接口中添加：

```typescript
  /** 发送 AI 编辑请求 */
  sendAIEdit: (action: AIAction, text: string) => Promise<void>
```

在 AppProvider 组件中实现：

```typescript
  const sendAIEdit = useCallback(
    async (action: AIAction, text: string) => {
      const prompt = buildAIPrompt(action, text)
      await send(prompt)
    },
    [send],
  )
```

在 `value` useMemo 中添加：

```typescript
      sendAIEdit,
```

- [ ] **Step 2: Commit**

```bash
git add src/context/AppProvider.tsx
git commit -m "feat(ai): AppProvider 添加 sendAIEdit 方法"
```

---

### Task 20: Phase 4 验收

- [ ] **Step 1: 功能验证**

- [ ] 选中文字可发送到 AI 面板
- [ ] AI 返回的修改可正确应用到编辑器
- [ ] 修订模式下 AI 修改标记为 tracked change
- [ ] 快捷键正常工作

---

## 附录

### 回退方案执行步骤

如果在任何 Phase 中需要回退到导入/导出方案：

1. 保留 `useDocxConverter` hook（用于导入/导出）
2. 移除 `DocxEditorView` 组件和相关样式
3. 在案件右键菜单中添加：
   - "导入 DOCX"：调用 `bufferToDocx` → `docxToMd` → `updateCaseDocument`
   - "导出 DOCX"：调用 `mdToDocx` → `docxToBuffer` → 下载文件
4. 移除 ProseMirror peer dependencies
5. 更新相关组件，移除 docxMode 相关逻辑

### 测试清单

#### 手动测试场景

| 场景 | 步骤 | 预期结果 |
|------|------|----------|
| 模式切换 | 打开文档 → 切换 DOCX → 编辑 → 切换 Markdown | 内容一致 |
| 修订追踪 | ReviewView → 开启修订 → 删除文字 → 看到删除线 | 标记正确显示 |
| AI 编辑 | 选中文字 → AI 润色 → 应用 | 文字被替换 |
| 大文档 | 打开 100 页说明书 → 滚动 → 编辑 | 流畅无卡顿 |
| 暗色模式 | 切换暗色主题 → 打开 DOCX 编辑器 | Chrome 适配暗色，页面白色 |
| 导入 DOCX | 上传 .docx 文件 | 内容正确显示为 Markdown |
| 导出 DOCX | 点击导出 | 下载 .docx 文件，格式正确 |

#### 回归测试

- [ ] 现有 Markdown 编辑功能不受影响
- [ ] 案件管理功能正常
- [ ] AI 对话功能正常
- [ ] 设置保存/加载正常

### 文档更新

实施完成后需要更新：
- [ ] `README.md`：添加 DOCX 编辑功能说明
- [ ] `DESKTOP-DEVELOPMENT-LOG.md`：记录本次变更
- [ ] 用户文档：添加 DOCX 模式使用指南

---

*本计划基于设计文档 `docs/superpowers/specs/2026-06-01-docx-editor-integration-design.md`*
