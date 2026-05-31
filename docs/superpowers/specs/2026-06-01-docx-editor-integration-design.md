# 桌面端 DOCX 编辑器集成设计方案

**日期**: 2026-06-01  
**作者**: YunXi Agent  
**状态**: 待审查  

---

## 1. 背景与目标

### 1.1 当前状态

云熙智能体桌面端（Tauri + React）已具备完整的专利工作流支持：
- 案件管理、文档组织（说明书、权利要求书、审查意见等）
- Markdown 格式编辑（DraftView、ClaimsView、ReviewView、CompareView、SearchView）
- AI 辅助（右侧对话面板）
- 材料导入/导出

### 1.2 目标

引入 `@eigenpal/docx-editor-react` WYSIWYG 编辑器，使桌面端能够：
- 直接以 docx 格式编辑专利文档
- 保留现有 Markdown 编辑能力（双模式）
- 支持修订追踪（tracked changes）
- AI 编辑辅助集成到右侧面板
- UI 风格与现有桌面端保持一致

### 1.3 非目标

- 不替换现有工作流引擎和案件管理系统
- 不改动后端 Rust 代码（纯前端集成）
- 不实现多人实时协作（docx-editor 支持但不在本次范围）

---

## 2. 需求确认

基于与用户的澄清讨论，最终确认的需求：

| 需求项 | 决策 |
|--------|------|
| 编辑方式 | 新增 DOCX 视图，同时保留 Markdown 编辑 |
| 文档类型 | 全部专利文档（说明书、权利要求书、审查意见答复、对比分析等） |
| AI 编辑 | 需要，集成到右侧面板 AI 助手 |
| 修订追踪 | 需要，审查意见答复必须支持修订模式 |
| UI 一致性 | 必须高度一致，否则回退到导入/导出方案 |

---

## 3. 方案决策

### 3.1 候选方案回顾

| 方案 | 描述 | 优点 | 缺点 |
|------|------|------|------|
| **A** | 安装新版 docx-editor-react，包装为新增视图 | 功能完整、开发周期短、官方维护 | 包体积增加、需样式覆盖 |
| **B** | 仅引入 docx-core headless + 自研 UI | UI 100% 统一、体积可控 | 开发周期 2-3 个月、学习成本极高 |
| **C** | 独立进程 + iframe 嵌入 | 完全隔离 | 体验割裂、IPC 桥接复杂 |

### 3.2 选定方案：方案 A

理由：
1. 专利桌面端的核心价值是 AI 辅助专利工作流，而非编辑器自研
2. docx-editor 已内置 tracked changes 和 DocumentAgent API，与需求高度匹配
3. 包体积增加（~2-3MB）在桌面端可接受
4. 可快速验证价值，后续如需深度定制再考虑方案 B

---

## 4. 架构设计

### 4.1 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    桌面端主窗口 (YunXi Desktop)              │
├─────────────────────────────────────────────────────────────┤
│  左侧资源管理器  │  中央工作区                    │ 右侧 AI  │
│                │  ┌─────────────────────────┐  │          │
│  案件/文档列表  │  │  视图切换 (Tab)          │  │  对话    │
│                │  │  ┌───────┐ ┌──────────┐ │  │  面板    │
│                │  │  │Markdown│ │ DOCX 模式 │ │  │          │
│                │  │  │  编辑  │ │ 编辑器    │ │  │          │
│                │  │  └───────┘ └──────────┘ │  │          │
│                │  │     ↑ 双向同步            │  │          │
│                │  └─────────────────────────┘  │          │
├─────────────────────────────────────────────────────────────┤
│  底部输出面板                                               │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 新增组件

| 组件 | 职责 | 位置 |
|------|------|------|
| `DocxEditorView` | docx-editor 包装组件，处理主题适配、AI 桥接 | `components/docx-editor/` |
| `DocxEditorToolbar` | 精简工具栏（隐藏默认标题栏，复用桌面端 breadcrumb） | `components/docx-editor/` |
| `ViewModeToggle` | Markdown ↔ DOCX 模式切换按钮 | `components/workbench/` |
| `useDocxConverter` | Markdown ↔ DOCX 双向转换 hook | `hooks/` |

### 4.3 现有组件修改

| 组件 | 修改内容 |
|------|----------|
| `EditorWorkbench` | 在视图渲染逻辑中增加 DOCX 模式分支 |
| `EditorTabBar` | 增加模式切换按钮 |
| `AppProvider` | 增加 `docxMode` 状态、文档格式缓存 |

---

## 5. UI 一致性策略

### 5.1 样式系统分析

**docx-editor 样式特点：**
- 底层 UI 组件（Button/Dialog/Select）采用 cva + Tailwind utilities，与 shadcn 风格一致
- CSS 变量被隔离在 `.ep-root` 选择器下，不会污染桌面端
- 存在硬编码颜色（`bg-white`, `bg-slate-900`）和 Google 蓝色强调色（`#1a73e8`）

**YunXi 桌面端样式特点：**
- 自定义暖色调主题（`#F5F2EE` 背景、`#4A7C6F` 强调色）
- CSS 变量定义在 `:root` 和 `.dark` 下
- 玻璃拟态效果（glassmorphism）

### 5.2 覆盖策略

| UI 元素 | 覆盖难度 | 方案 | 预期效果 |
|---------|----------|------|----------|
| 按钮、输入框、对话框 | 低 | 覆盖 `.ep-root` 下的 shadcn CSS 变量 | 自动适配主题 |
| 工具栏背景 | 中 | CSS 覆盖 `.ep-root .bg-white` → `var(--bg-elevated)` | 融入桌面端 |
| 编辑器页面（纸张） | **保持白色** | 不覆盖 | WYSIWYG 真实预览 |
| Tooltip、ContextMenu | 中 | 额外 CSS 覆盖背景色/文字色 | 风格统一 |
| 选择高亮 | 低 | 覆盖 selection 颜色变量 | 使用桌面端强调色 |

### 5.3 回退策略

如果在 Phase 1 中发现样式覆盖成本过高（需覆盖超过 50 个硬编码样式类，或关键交互组件无法适配），则**立即回退到备选方案**：
- 保留现有 Markdown 编辑
- 仅增加 docx 导入/导出功能（通过 `docx-core` 的 `parseDocx` / `serializeDocx`）

---

## 6. 数据流设计

### 6.1 文档生命周期

```
案件文档存储 (Markdown)
    │
    ├─→ 现有视图 (DraftView/ClaimsView/ReviewView)
    │      → Markdown 直接编辑
    │      → updateCaseDocument(docId, markdown)
    │
    └─→ DocxEditorView (当切换为 DOCX 模式)
           │
           ├─→ 首次加载：md → docx
           │      → createDocumentWithText(markdown)
           │      → 渲染为 WYSIWYG
           │
           ├─→ 用户编辑
           │      → onChange 回调
           │      → debounce 500ms
           │      → docx → md (serializeDocx + 文本提取)
           │      → updateCaseDocument(docId, newMd)
           │
           └─→ 保存/切换回 Markdown 模式
                  → 立即同步 docx → md
                  → updateCaseDocument(docId, newMd)
```

### 6.2 AI 编辑桥接

```
用户在 DOCX 编辑器中选中文字
    │
    ├─→ 右键菜单 / 快捷键触发 "AI 编辑"
    │
    ├─→ DocumentAgent API 获取选中内容上下文
    │      → buildSelectionContext()
    │
    ├─→ 桥接到右侧 AI 面板
    │      → 发送选中内容 + 上下文到 AI
    │      → 显示在 AI 对话中
    │
    ├─→ AI 返回编辑建议
    │      → 在 AI 面板中显示 diff
    │      → 用户点击 "应用"
    │
    └─→ DocumentAgent 执行修改
           → executeCommand(ReplaceTextCommand)
           → 若开启修订模式：应用为 tracked change
           → onChange 触发 → 同步回 Markdown
```

### 6.3 修订追踪模式

```
审查意见答复场景
    │
    ├─→ 用户开启 "修订模式"（类似 Word 的 Track Changes）
    │
    ├─→ 所有编辑操作自动标记为 tracked change
    │      → insertions: <w:ins>
    │      → deletions: <w:del>
    │
    ├─→ 用户可查看/接受/拒绝修订
    │      → docx-editor 内置 UI 支持
    │
    └─→ 导出时保留修订标记
           → 生成带 tracked changes 的 docx 文件
           → 或接受所有修订后导出清洁版
```

---

## 7. 实施计划

### Phase 1: 基础集成与主题适配（2-3 天）

**目标**: 安装 docx-editor，验证 UI 一致性可行性

**任务**:
- [ ] 安装 `@eigenpal/docx-editor-react` 及 peer dependencies
- [ ] 创建 `DocxEditorView` 基础包装组件
- [ ] 编写主题覆盖 CSS（`docx-editor-theme-overrides.css`）
- [ ] 验证关键 UI 组件（工具栏、对话框、右键菜单）风格一致性
- [ ] **决策检查点**: 若样式覆盖成本过高，回退到导入/导出方案

**验证标准**:
- [ ] 编辑器在桌面端正常渲染
- [ ] 工具栏颜色与桌面端主题一致
- [ ] 对话框、下拉菜单风格统一
- [ ] 无样式污染（不影响桌面端其他组件）

**退出条件**: UI 一致性达到可接受水平，或明确回退决策

### Phase 2: DraftView DOCX 模式试点（3-4 天）

**目标**: 在说明书视图中集成 DOCX 编辑

**任务**:
- [ ] 修改 `EditorWorkbench` 支持视图模式切换
- [ ] 实现 `useDocxConverter` hook（md ↔ docx）
- [ ] 在 `DraftView` 中集成 `DocxEditorView`
- [ ] 实现模式切换状态持久化（localStorage）
- [ ] 测试文档打开、编辑、保存全流程

**验证标准**:
- [ ] 说明书可在 Markdown 和 DOCX 模式间切换
- [ ] DOCX 模式编辑后，Markdown 内容正确同步
- [ ] 切换模式不丢失内容
- [ ] 性能可接受（大文档 >100 页不卡顿）

**退出条件**: DraftView DOCX 模式稳定可用

### Phase 3: 多视图扩展与修订追踪（4-5 天）

**目标**: 支持全部文档类型，启用修订模式

**任务**:
- [ ] 扩展 `ClaimsView` 支持 DOCX 模式
- [ ] 扩展 `ReviewView` 支持 DOCX 模式 + tracked changes
- [ ] 扩展 `CompareView`、`SearchView` 支持 DOCX 模式
- [ ] 实现修订模式开关（全局状态）
- [ ] 修订标记的显示/接受/拒绝 UI
- [ ] 测试审查意见答复场景

**验证标准**:
- [ ] 全部视图支持 DOCX 模式切换
- [ ] 审查意见答复可开启修订模式
- [ ] 修订标记正确显示
- [ ] 接受/拒绝修订后内容正确更新

**退出条件**: 全部视图 DOCX 模式可用，修订追踪功能正常

### Phase 4: AI 编辑桥接（3-4 天）

**目标**: 编辑器内选中文字可通过右侧 AI 面板处理

**任务**:
- [ ] 实现 `aiBridge` 模块（编辑器 ↔ AI 面板通信）
- [ ] 右键菜单增加 "AI 润色"/"AI 扩写"/"AI 改写" 选项
- [ ] 选中文字发送到 AI 面板的逻辑
- [ ] AI 返回后应用修改（通过 DocumentAgent API）
- [ ] 支持修订模式下的 AI 修改（标记为 tracked change）
- [ ] 快捷键绑定（如 ⌘+Shift+A 唤起 AI）

**验证标准**:
- [ ] 选中文字可发送到 AI 面板
- [ ] AI 返回的修改可正确应用到编辑器
- [ ] 修订模式下 AI 修改标记为 tracked change
- [ ] 快捷键正常工作

**退出条件**: AI 编辑桥接功能完整可用

---

## 8. 技术细节

### 8.1 依赖清单

```json
{
  "dependencies": {
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
  }
}
```

### 8.2 关键 API 使用

```typescript
// DOCX 解析
import { parseDocx } from '@eigenpal/docx-editor-react/core';

// DOCX 序列化
import { serializeDocx } from '@eigenpal/docx-editor-react/core/serializer';

// 创建空文档
import { createDocumentWithText } from '@eigenpal/docx-editor-react/core/utils';

// DocumentAgent API
import { DocumentAgent, executeCommand } from '@eigenpal/docx-editor-react/core/agent';

// React 组件
import { DocxEditor, type DocxEditorRef } from '@eigenpal/docx-editor-react';
```

### 8.3 文件映射

| 新建文件 | 预估行数 | 职责 |
|----------|----------|------|
| `components/docx-editor/DocxEditorView.tsx` | ~200 | docx-editor 包装组件 |
| `components/docx-editor/DocxEditorToolbar.tsx` | ~150 | 精简工具栏 |
| `components/docx-editor/docx-editor-theme-overrides.css` | ~100 | 主题覆盖样式 |
| `components/workbench/ViewModeToggle.tsx` | ~80 | 模式切换按钮 |
| `hooks/useDocxConverter.ts` | ~120 | md ↔ docx 转换 |
| `utils/aiBridge.ts` | ~150 | AI 编辑桥接 |

| 修改文件 | 修改内容 |
|----------|----------|
| `components/workbench/EditorWorkbench.tsx` | 增加 DOCX 模式渲染分支 |
| `components/workbench/EditorTabBar.tsx` | 增加模式切换按钮 |
| `context/AppProvider.tsx` | 增加 `docxMode` 状态 |
| `package.json` | 增加 docx-editor 依赖 |

---

## 9. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| UI 一致性无法达标 | 高 | Phase 1 设置明确退出检查点，不达标立即回退 |
| 包体积过大影响启动 | 中 | 使用 dynamic import 懒加载 docx-editor |
| Markdown ↔ DOCX 转换失真 | 高 | 全面测试专利文档格式（标题、列表、表格、公式） |
| 性能问题（大文档卡顿） | 中 | Phase 2 进行性能基准测试，必要时虚拟滚动 |
| docx-editor 新版 API 变更 | 中 | 锁定版本，升级前充分测试 |

---

## 10. 验证策略

### 10.1 功能验证

| 场景 | 验证步骤 |
|------|----------|
| 模式切换 | 打开说明书 → 切换 DOCX 模式 → 编辑 → 切换回 Markdown → 内容一致 |
| 修订追踪 | 开启修订模式 → 删除一段文字 → 看到删除标记 → 接受修订 → 内容更新 |
| AI 编辑 | 选中文字 → 右键 AI 润色 → AI 面板显示建议 → 应用 → 内容更新 |
| 大文档 | 打开 100+ 页说明书 → 滚动流畅 → 编辑无卡顿 |

### 10.2 UI 验证

| 检查项 | 通过标准 |
|--------|----------|
| 工具栏颜色 | 与桌面端 `--bg-elevated` / `--accent-primary` 一致 |
| 对话框风格 | 圆角、阴影、边框与桌面端 shadcn 对话框一致 |
| 右键菜单 | 背景色、文字色、hover 效果与桌面端一致 |
| 暗色模式 | 编辑器 chrome 适配暗色主题 |

---

## 11. 回退方案

如果在任何阶段遇到不可克服的障碍，执行以下回退：

1. **保留现有 Markdown 编辑不变**
2. **增加 docx 导入/导出功能**：
   - 导入：用户上传 .docx 文件 → `parseDocx` → Markdown → 保存到案件
   - 导出：案件文档 Markdown → `createDocumentWithText` → .docx 文件下载
3. **移除 docx-editor 依赖**，减少包体积

---

## 12. 附录

### 12.1 参考文档

- [docx-editor 官方文档](https://www.docx-editor.dev/docs)
- [docx-editor 迁移指南](https://www.docx-editor.dev/docs/latest/migration)（0.x → 1.x）
- [ProseMirror 指南](https://prosemirror.net/docs/guide/)

### 12.2 相关代码

- 桌面端前端: `rust/crates/yunxi-cli/frontend/src/`
- 现有编辑器视图: `src/components/center/{DraftView,ClaimsView,ReviewView}.tsx`
- 状态管理: `src/context/AppProvider.tsx`

---

*本方案经过与用户讨论确认，用户已批准实施。*
