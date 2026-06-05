# TUI 组件清单

> 本文档记录 YunXi TUI 所有组件状态，与实现保持同步。
> 最后更新：2026-06-04（Phase A 完成）

## 组件体系

采用**桥接模式**：
- **公共接口**：`Component` trait（生命周期管理、事件处理）
- **内部实现**：基于 `ratatui::widgets::Widget`（渲染）
- **新组件规范**：实现 `Component` trait，内部用 `Widget` 渲染

## 组件状态

### ✅ 已启用（编译通过 + 测试通过）

| 组件 | 文件 | 用途 | 测试覆盖 |
|------|------|------|---------|
| `Alert` | `alert.rs` | 告警提示 | ✅ |
| `Breadcrumb` | `breadcrumb.rs` | 面包屑导航 | ✅ |
| `Button` | `button.rs` | 按钮 | ✅ |
| `Collapsible` | `collapsible.rs` | 折叠面板 | ✅ 已修复（Arc+手动Debug） |
| `CommandPalette` | `command_palette.rs` | 命令面板 | ⚠️ 骨架，待实现搜索 |
| `Confirm` | `confirm.rs` | 确认对话框 | ✅ |
| `ErrorDialog` | `error_dialog.rs` | 错误详情弹窗 | ✅ 已修复 |
| `Form` | `form.rs` | 表单输入 | ✅ 已修复 |
| `Input` | `input.rs` | 文本输入框 | ✅ |
| `KeymapEditor` | `keymap_editor.rs` | 快捷键编辑器 | ✅ 已修复 |
| `Label` | `label.rs` | 标签文本 | ✅ |
| `Layout` | `layout.rs` | 弹性布局 | ✅ |
| `List` | `list.rs` | 列表选择 | ✅ |
| `Menu` | `menu.rs` | 菜单 | ✅ |
| `Modal` | `modal.rs` | 模态框 | ✅ |
| `Picker` | `picker.rs` | 选择器 | ✅ |
| `ProgressBar` | `progress_bar.rs` | 进度条 | ✅ |
| `ProgressIndicator` | `progress_indicator.rs` | 进度指示器 | ✅ 已解除注释 |
| `Sidebar` | `sidebar.rs` | 侧边栏导航 | ✅ 待集成到主布局 |
| `Spacer` | `spacer.rs` | 间距占位 | ✅ |
| `Spinner` | `spinner.rs` | 加载动画 | ✅ |
| `Tab` | `tab.rs` | 标签页 | ✅ |
| `Table` | `table.rs` | 表格 | ✅ |
| `ThinkingBlock` | `thinking_block.rs` | 思考过程展示 | ✅ 已解除注释 |
| `Toast` | `toast.rs` | 轻提示 | ✅ |
| `Tree` | `tree.rs` | 树形结构 | ✅ |

### 🔧 widgets/ 目录（ratatui 直接实现）

| 组件 | 文件 | 用途 | 状态 |
|------|------|------|------|
| `ChatView` | `chat_view_ratatui.rs` | 聊天主视图 | ✅ 运行中 |
| `CommandPaletteRatatui` | `command_palette_ratatui.rs` | 命令面板（ratatui版） | ⚠️ 骨架 |
| `FlowHITLOverlay` | `flow_hitl_overlay_ratatui.rs` | HITL流程覆盖层 | ✅ |
| `GuideOverlay` | `guide_overlay_ratatui.rs` | 引导覆盖层 | ✅ |
| `HelpOverlay` | `help_overlay_ratatui.rs` | 帮助覆盖层 | ✅ |
| `InputBar` | `input_bar_ratatui.rs` | 输入栏 | ✅ |
| `MessageBubble` | `message_bubble.rs` | 消息气泡 | ✅ |
| `PermissionOverlay` | `permission_overlay_ratatui.rs` | 权限覆盖层 | ✅ |
| `SessionPicker` | `session_picker_ratatui.rs` | 会话选择器 | ✅ |
| `StatusBar` | `status_bar_ratatui.rs` | 状态栏 | ✅ |
| `TitleBar` | `title_bar.rs` | 标题栏 | ✅ |
| `ToolBlock` | `tool_block.rs` | 工具调用块 | ✅ |
| `ToolPanel` | `tool_panel_ratatui.rs` | 工具面板 | ✅ |

## Phase A 变更记录

### 已修复组件（解除注释 + 编译修复）

1. **Collapsible** (`collapsible.rs`)
   - 修复：`Box<dyn Fn>` → `Arc<dyn Fn>`（支持Clone）
   - 修复：手动实现 `Debug` trait（跳过callback字段）
   - 状态：✅ 编译通过，测试通过

2. **ThinkingBlock** (`thinking_block.rs`)
   - 修复：`Widget::render(&collapsible)` → `collapsible.clone().render()`（满足Widget的self传参）
   - 状态：✅ 编译通过，测试通过

3. **ProgressIndicator** (`progress_indicator.rs`)
   - 变更：解除 `components/mod.rs` 注释
   - 状态：✅ 编译通过，已有测试

4. **ErrorDialog** (`error_dialog.rs`)
   - 变更：解除编译绕过标记
   - 状态：✅ 编译通过，已有测试

5. **Form** (`form.rs`)
   - 变更：解除编译绕过标记
   - 状态：✅ 编译通过，已有测试

6. **KeymapEditor** (`keymap_editor.rs`)
   - 变更：解除编译绕过标记
   - 状态：✅ 编译通过，已有测试

### 体系统一决策

**决策**：保留 Component trait 作为公共接口，内部基于 ratatui Widget 实现。

**理由**：
- Component trait 提供统一的生命周期（`on_mount`/`on_unmount`）、状态管理和事件处理接口
- ratatui Widget 提供高效渲染和生态兼容性
- 现有代码已大量采用此模式，迁移成本过高

**规范**：
- 新组件必须实现 `Component` trait
- 渲染逻辑使用 `ratatui::widgets::Widget` 或 `StatefulWidget`
- 避免在组件内部混合两套体系

## 下一步（Phase B）

详见 [TUI-IMPROVEMENT-PLAN.md](./TUI-IMPROVEMENT-PLAN.md) Phase B 部分。
