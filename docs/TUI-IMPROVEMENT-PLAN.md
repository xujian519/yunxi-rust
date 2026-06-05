# YunXi TUI 改进实施计划

**计划日期**：2026-06-04  
**基于审阅**：`/Users/xujian/projects/YunXi/docs/TUI-REVIEW-REPORT.md`  
**目标**：将 TUI 与 opencode 等业界优秀实践对齐，综合对齐度从 30% 提升至 90%+  
**预计周期**：6-8 周  
**状态**：🟢 Phase A 已完成，进入 Phase B 执行

---

## 一、目标与范围

### 1.1 核心目标

| 维度 | 当前状态 | 目标状态 | 衡量标准 |
|------|---------|---------|---------|
| **功能完整性** | 4/20 项完成 | 18/20 项完成 | 功能清单逐项验收 |
| **与 opencode 对齐度** | ~30% | ≥90% | 16 项功能对标检查 |
| **组件质量** | 两套体系并存，大量注释/编译错误 | 统一体系，零编译错误 | `cargo build` 零警告 |
| **测试覆盖** | 核心框架有测试，组件测试薄弱 | 核心+关键组件均有测试 | 新增 50+ 单元测试 |
| **文档同步** | 设计文档与实现脱节 | 文档与代码实时同步 | 每个 PR 同步更新文档 |

### 1.2 不在本次范围内

- 桌面端 GUI（Tauri + React）—— 并行发展，不在 TUI 改进范围
- 底层 LLM 调用逻辑 —— 仅涉及 UI 展示层
- 专利领域业务逻辑 —— 仅涉及通用 TUI 框架

---

## 二、当前问题清单（按优先级排序）

### 🔴 P0 — 阻塞性问题

| # | 问题 | 影响 | 涉及文件 |
|---|------|------|---------|
| 1 | 两套组件体系未统一（Component trait vs ratatui Widget） | 架构债务，新增功能难以选择体系 | `components/`, `widgets/` |
| 2 | `collapsible`、`thinking_block`、`progress_indicator` 被注释 | 功能缺失，代码腐烂 | `components/mod.rs` |
| 3 | `error_dialog`、`form`、`keymap_editor` 编译错误 | 无法启用相关功能 | `components/` |
| 4 | `CommandPalette` 仅骨架，无法搜索执行 | 核心交互缺失 | `widgets/command_palette_ratatui.rs` |
| 5 | `Sidebar` 未集成到主布局 | 导航系统缺失 | `components/sidebar.rs`, `app_ratatui.rs` |

### 🟡 P1 — 重要体验问题

| # | 问题 | 影响 | 涉及文件 |
|---|------|------|---------|
| 6 | 工具输出无折叠/展开交互 | 工具调用体验差 | `widgets/tool_panel_ratatui.rs` |
| 7 | 思考过程（reasoning_delta）不可见 | 用户无法理解 AI 推理 | `components/thinking_block.rs`（被注释） |
| 8 | 代码语法高亮未集成到流式输出 | 代码可读性差 | `syntax/highlighter.rs`, `widgets/message_bubble.rs` |
| 9 | 快捷键系统仅有基础 keymap，无自定义 | 高级用户效率低 | `keymap/` |
| 10 | 主题仅 2 套，无切换 UI | 个性化不足 | `theme/presets.rs`, `theme/manager.rs` |
| 11 | `Arc<Mutex<GlobalState>>` 粒度过粗 | 高频事件性能瓶颈 | `core/app.rs` |

### 🟢 P2 — 增强项

| # | 问题 | 影响 | 涉及文件 |
|---|------|------|---------|
| 12 | 文件差异可视化渲染未完成 | diff 体验差 | `diff/`, `components/diff_view.rs` |
| 13 | 无文本选择+复制功能 | 无法复制对话内容 | `clipboard/` |
| 14 | 工作区管理视图未完成 | 文件树缺失 | `workspace/` |
| 15 | `ChatView` 无虚拟滚动 | 长对话性能下降 | `widgets/chat_view_ratatui.rs` |
| 16 | 无终端兼容性测试 | tmux/SSH/Windows Terminal 可能异常 | — |

---

## 三、实施阶段

### Phase A：质量清理与架构统一（第 1-2 周）

**目标**：消除技术债务，统一组件体系，确保代码基线健康

#### 任务 A1：组件体系统一决策

**描述**：评估并决定保留 Component trait 体系还是 ratatui Widget 直接实现体系，或建立桥接层。

**验收标准**：
- [ ] 产出《组件体系统一决策文档》（≤2 页）
- [ ] 明确所有现有组件的迁移路径
- [ ] 建立组件开发规范（新组件应遵循哪种体系）

**决策参考**：
- Component trait 优势：生命周期管理（`on_mount`/`on_unmount`）、状态封装、可测试性
- ratatui Widget 优势：与 ratatui 生态无缝集成、性能更优、代码更简洁
- 推荐方案：保留 Component trait 作为公共接口，内部实现基于 ratatui Widget（桥接模式）

**预估工作量**：3 天  
**负责人**：架构负责人

#### 任务 A2：修复被注释组件

**描述**：逐一修复 `collapsible`、`thinking_block`、`progress_indicator`，使其编译通过并运行正常。

**验收标准**：
- [ ] `collapsible`：支持折叠/展开动画，可配置标题和图标
- [ ] `thinking_block`：支持多步思考展示，可折叠，自动滚动
- [ ] `progress_indicator`：支持 spinner、进度条、百分比三种模式
- [ ] 每个组件均有 `TestBackend` 测试，验证渲染不 panic
- [ ] `components/mod.rs` 中解除注释

**依赖**：A1（确定组件体系后实施）  
**预估工作量**：4 天  
**负责人**：组件开发者

#### 任务 A3：修复编译错误组件

**描述**：修复 `error_dialog`、`form`、`keymap_editor` 的编译错误。

**验收标准**：
- [ ] `error_dialog`：正确显示错误信息，支持重试/取消操作
- [ ] `form`：支持文本输入、下拉选择、复选框等字段类型
- [ ] `keymap_editor`：支持按键录制、冲突检测、保存配置
- [ ] 解除 `components/mod.rs` 中的编译绕过（`#![allow(dead_code)]` 可移除）

**依赖**：A1  
**预估工作量**：3 天  
**负责人**：组件开发者

#### 任务 A4：补全关键组件单元测试

**描述**：为核心组件和修复后的组件补充测试。

**验收标准**：
- [ ] `components/tests.rs` 覆盖所有修复后的组件
- [ ] 每个组件至少 3 个测试用例：正常渲染、边界条件、事件处理
- [ ] 测试通过 `cargo test --workspace`

**预估工作量**：2 天  
**负责人**：QA/测试开发者

#### 任务 A5：文档对齐

**描述**：更新 `TUI-ENHANCEMENT-PLAN.md` 和开发日志，确保与实际代码一致。

**验收标准**：
- [ ] `TUI-ENHANCEMENT-PLAN.md` 中过时内容已标注或更新
- [ ] 新增 `COMPONENTS.md`，记录每个组件的用途、接口、使用示例
- [ ] `TUI-DEVELOPMENT-LOG.md` 记录 Phase A 变更

**预估工作量**：1 天  
**负责人**：技术写作者

**Phase A 里程碑**：
- ✅ `cargo build --workspace` 零警告
- ✅ `cargo test --workspace` 全绿
- ✅ 所有组件可编译、可运行
- ✅ 组件体系统一文档完成

---

### Phase B：核心功能实现（第 3-5 周）

**目标**：实现 opencode 核心交互功能，提升用户体验

#### 任务 B1：命令面板完整功能 ✅

**描述**：实现 Ctrl+P 命令面板，支持命令搜索、快捷键提示、快速执行。

**验收标准**：
- [x] 按 `Ctrl+P` 唤起命令面板，覆盖全屏半透明遮罩
- [x] 支持模糊搜索（命令名、描述、快捷键）
- [x] 上下箭头选择，`Enter` 执行，`Esc` 关闭
- [x] 命令分类显示（文件、编辑、视图、工具等）
- [x] 与现有斜杠命令系统联动（`/status`、`/clear` 等）

**技术要点**：
- 复用 `widgets/command_palette_ratatui.rs` 骨架
- 命令数据来源：`commands/` crate 的斜杠命令 + TUI 专用命令（切换主题、显示侧边栏等）
- 搜索算法：前缀匹配 + 模糊匹配（fuse-rs 或简单实现）

**预估工作量**：5 天  
**负责人**：核心功能开发者

#### 任务 B2：工具输出折叠/展开 ✅

**描述**：为工具调用输出添加折叠/展开交互，优化长工具输出的可读性。

**验收标准**：
- [x] 工具调用默认折叠，显示工具名 + 状态图标
- [x] 点击或按 `Enter`/`Shift+Enter` 展开，显示完整输出
- [x] 键盘导航支持（↑↓ 在工具面板中切换焦点）
- [x] 支持一键展开/折叠所有工具（`Ctrl+E`）
- [ ] 代码块在工具输出中支持语法高亮（依赖 B4）

**技术要点**：
- 复用修复后的 `components/collapsible.rs`
- 与 `widgets/tool_panel_ratatui.rs` 集成
- 新增 `tool_panel_focus_index` 支持键盘导航
- 智能折叠：输出 ≤10 行时自动展开，>10 行时折叠

**预估工作量**：4天
**负责人**：组件开发者

**完成状态**：2026-06-04
- `ToolEntry::new()` 默认折叠
- `ToolBlock` 新增 `focused` 状态渲染（高亮背景 + ▸ 指示器）
- `Ctrl+E` 切换所有工具折叠/展开
- `Enter`/`Shift+Enter` 切换当前焦点工具
- ↑↓ 键在工具面板中导航焦点
- 新增 6 个单元测试

#### 任务 B3：思考过程可视化

**描述**：展示 AI 的 reasoning_delta（思考过程），支持折叠和流式更新。

**验收标准**：
- [ ] 思考过程以独立区块显示，与最终回答区分
- [ ] 默认折叠，显示思考步数和耗时
- [ ] 展开后显示完整思考链
- [ ] 支持流式更新（思考过程中实时显示）
- [ ] 可配置是否自动展开

**技术要点**：
- 复用修复后的 `components/thinking_block.rs`
- 数据流：`api` crate 的 reasoning_delta → `tui` 状态更新 → 渲染
- 需修改 `widgets/message_bubble.rs` 支持 thinking block 嵌入

**预估工作量**：4 天  
**负责人**：组件开发者

#### 任务 B4：代码语法高亮集成

**描述**：将 `syntax/highlighter.rs` 集成到流式消息输出中，实现实时代码高亮。

**验收标准**：
- [ ] 代码块自动识别语言（基于 markdown 标签或启发式检测）
- [ ] 使用 `syntect` 进行语法高亮，主题与 TUI 主题一致
- [ ] 流式输出时，代码块缓存并延迟高亮（避免频繁重渲染）
- [ ] 支持常见语言：Rust、Python、JavaScript、TypeScript、Markdown、JSON、YAML

**技术要点**：
- `syntax/highlighter.rs` 已有基础实现，需完善语言检测和主题映射
- 与 `widgets/message_bubble.rs` 的代码块渲染逻辑集成
- 性能考虑：高亮结果缓存，避免重复解析

**预估工作量**：3 天  
**负责人**：组件开发者

#### 任务 B5：侧边栏导航集成

**描述**：将 `components/sidebar.rs` 集成到主布局，提供会话列表、工具面板、设置入口。

**验收标准**：
- [ ] 侧边栏可折叠/展开（默认折叠，按 `Ctrl+B` 切换）
- [ ] 显示会话列表（当前会话高亮）
- [ ] 显示工具状态（运行中/已完成）
- [ ] 显示快捷操作（新建会话、清空对话、设置）
- [ ] 侧边栏宽度可调（鼠标拖拽或键盘）
- [ ] 与主聊天区布局不冲突

**技术要点**：
- 修改 `app_ratatui.rs` 的主布局逻辑
- `Sidebar` 组件需支持折叠状态持久化
- 会话列表数据来自 `session/manager.rs`

**预估工作量**：5 天  
**负责人**：核心功能开发者

**Phase B 里程碑**：
- ✅ `Ctrl+P` 命令面板可用
- ✅ 工具输出可折叠展开
- ✅ 思考过程可见
- ✅ 代码块有语法高亮
- ✅ 侧边栏可导航

---

### Phase C：交互增强（第 6-7 周）

**目标**：完善个性化和高级交互功能

#### 任务 C1：快捷键自定义配置

**描述**：实现 `keymap.toml` 加载和快捷键自定义。

**验收标准**：
- [ ] 支持 `~/.yunxi/keymap.toml` 自定义快捷键
- [ ] 快捷键可绑定到任意 Action（命令、导航、工具调用等）
- [ ] 冲突检测：自定义快捷键与默认快捷键冲突时提示
- [ ] 提供 `/keymap` 命令快速编辑配置
- [ ] 配置热加载（无需重启生效）

**技术要点**：
- 复用修复后的 `components/keymap_editor.rs`
- 与 `keymap/` 模块集成
- TOML 解析使用 `toml` crate

**预估工作量**：4 天  
**负责人**：核心功能开发者

#### 任务 C2：多主题切换 UI

**描述**：实现主题切换 UI 和主题编辑器。

**验收标准**：
- [ ] 命令面板或侧边栏可切换主题（默认暗色/亮色 + 自定义主题）
- [ ] 主题切换即时生效，无需重启
- [ ] 支持自定义主题配置（`~/.yunxi/themes/`）
- [ ] 主题编辑器：可视化调整颜色并预览

**技术要点**：
- 复用 `theme/manager.rs` 和 `theme/presets.rs`
- 新增 `Theme::load_from_file()` 和 `Theme::save_to_file()`
- 主题切换时全量重渲染

**预估工作量**：4 天  
**负责人**：组件开发者

#### 任务 C3：文件差异可视化

**描述**：完善 `diff/` 和 `components/diff_view.rs`，实现彩色 diff 展示。

**验收标准**：
- [ ] diff 以行内高亮显示（绿色新增、红色删除、黄色修改）
- [ ] 支持 unified diff 和 side-by-side 两种模式
- [ ] 支持行号显示
- [ ] 支持在 diff 中跳转（上一处/下一处变更）

**技术要点**：
- `diff/parser.rs` 已有解析器，需完善渲染逻辑
- 与 `components/diff_view.rs` 集成
- 颜色方案与主题系统联动

**预估工作量**：3 天  
**负责人**：组件开发者

#### 任务 C4：文本选择与复制

**描述**：实现对话内容的文本选择和复制功能。

**验收标准**：
- [ ] 支持鼠标拖拽选择文本
- [ ] 支持键盘选择（`Shift+方向键`）
- [ ] `Ctrl+C` 复制选中内容到剪贴板
- [ ] 支持复制单条消息（右键菜单或快捷键）
- [ ] 支持复制整个对话（`/export` 命令增强）

**技术要点**：
- 复用 `clipboard/manager.rs`
- 与 `widgets/chat_view_ratatui.rs` 的文本渲染集成
- 剪贴板跨平台支持（`arboard` crate）

**预估工作量**：4 天  
**负责人**：组件开发者

#### 任务 C5：工作区文件树视图

**描述**：实现工作区文件树视图，支持浏览和打开文件。

**验收标准**：
- [ ] 侧边栏或独立面板显示工作区文件树
- [ ] 支持展开/折叠目录
- [ ] 选中文件可查看内容（集成到聊天区或新面板）
- [ ] 文件图标根据类型显示不同符号

**技术要点**：
- 复用 `components/tree.rs`
- 与 `workspace/manager.rs` 集成
- 文件读取使用 `tools/` crate 的 ReadFile

**预估工作量**：3 天  
**负责人**：组件开发者

**Phase C 里程碑**：
- ✅ 快捷键可自定义
- ✅ 主题可切换和编辑
- ✅ diff 彩色可视化
- ✅ 文本可选择和复制
- ✅ 工作区文件树可用

---

### Phase D：性能优化与打磨（第 8 周）

**目标**：解决性能瓶颈，提升稳定性，确保兼容性

#### 任务 D1：ChatView 虚拟滚动

**描述**：为长对话实现虚拟滚动，只渲染可见区域的消息。

**验收标准**：
- [ ] 1000+ 条消息的对话流畅滚动（不卡顿）
- [ ] 内存占用不随消息数量线性增长
- [ ] 滚动条位置准确
- [ ] 支持跳转到首条/末条消息

**技术要点**：
- 使用 ratatui 的 `List` widget 虚拟化或自定义实现
- 消息高度预估（固定高度 vs 动态高度）
- 与流式输出的滚动跟随逻辑兼容

**预估工作量**：4 天  
**负责人**：性能优化开发者

#### 任务 D2：状态管理优化

**描述**：将 `Arc<Mutex<GlobalState>>` 替换为细粒度状态管理。

**验收标准**：
- [ ] 状态按模块拆分（UI 状态、会话状态、主题状态等）
- [ ] 使用 `RwLock` 或通道（channel）替代 `Mutex`，减少锁竞争
- [ ] 高频更新（鼠标移动、流式文本）不阻塞主线程
- [ ] 基准测试：1000 次状态更新耗时 < 16ms（一帧）

**技术要点**：
- 参考 `redux` 或 `elm` 架构，Action → Reducer → 状态更新
- 使用 `tokio::sync::RwLock` 或 `parking_lot`
- 考虑使用事件总线（`tokio::sync::broadcast`）

**预估工作量**：3 天  
**负责人**：架构负责人

#### 任务 D3：渲染帧率优化

**描述**：优化渲染性能，实现差分渲染。

**验收标准**：
- [ ] 空闲时 CPU 占用 < 1%
- [ ] 仅变更区域重渲染，避免全屏刷新
- [ ] 动画（spinner、进度条）使用定时器触发，而非每帧重绘
- [ ] 使用 ratatui 的 `Frame::buffer_mut()` 局部更新

**技术要点**：
- 引入脏区域标记（dirty rect）
- 组件级 `should_render()` 判断
- 与 ratatui 的 `Widget::render` 集成

**预估工作量**：2 天  
**负责人**：性能优化开发者

#### 任务 D4：终端兼容性测试

**描述**：在多种终端环境下测试 TUI 兼容性。

**验收标准**：
- [ ] iTerm2 / Terminal.app（macOS）全部功能正常
- [ ] tmux 会话中渲染正常（无残影、颜色正确）
- [ ] SSH 远程会话中功能正常
- [ ] Windows Terminal / WSL 基本功能正常
- [ ] 256 色/真彩色终端自动适配

**技术要点**：
- 使用 `crossterm` 的终端能力检测
- 颜色回退（true color → 256 color → 16 color）
- 鼠标事件在不同终端的一致性

**预估工作量**：2 天  
**负责人**：QA/测试开发者

**Phase D 里程碑**：
- ✅ 长对话（1000+ 消息）流畅运行
- ✅ CPU 空闲占用 < 1%
- ✅ 多终端环境兼容
- ✅ 全量回归测试通过

---

## 四、验证标准

### 4.1 每阶段出口检查清单

```markdown
## 阶段出口检查

### 构建检查
- [ ] `cargo fmt` 无变更
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 通过
- [ ] `cargo test --workspace` 全绿
- [ ] `cargo build --release` 成功

### 功能检查
- [ ] 本阶段新增功能清单全部验收通过
- [ ] 上一阶段功能无回归（手动验证核心路径）
- [ ] 新增功能有对应测试覆盖

### 文档检查
- [ ] `TUI-DEVELOPMENT-LOG.md` 已更新
- [ ] 新增/修改的组件有 API 文档注释
- [ ] 用户可见的变更已记录（快捷键、命令等）
```

### 4.2 最终验收标准

| 检查项 | 标准 | 验证方式 |
|--------|------|---------|
| opencode 对齐度 | ≥90%（16/18 项功能对标） | 功能对照表逐项检查 |
| 编译 cleanliness | 零 warning | `cargo clippy -- -D warnings` |
| 测试通过率 | 100% | `cargo test --workspace` |
| 性能基准 | 1000 消息对话滚动流畅 | 手动测试 |
| 终端兼容性 | 5 种终端环境无异常 | 兼容性测试矩阵 |

---

## 五、里程碑与检查点

```
Week 1-2  [Phase A]  质量清理
    ├─ Day 3:  组件体系统一决策评审
    ├─ Day 7:  被注释组件修复完成
    └─ Day 10: 所有组件编译通过，测试补全

Week 3-5  [Phase B]  核心功能
    ├─ Day 17: 命令面板可用（内部演示）
    ├─ Day 24: 工具折叠 + 思考展示 + 代码高亮完成
    └─ Day 31: 侧边栏集成，Phase B 功能验收

Week 6-7  [Phase C]  交互增强
    ├─ Day 38: 快捷键自定义 + 主题切换完成
    ├─ Day 45: diff + 复制 + 工作区完成
    └─ Day 48: Phase C 功能验收

Week 8    [Phase D]  性能与打磨
    ├─ Day 52: 虚拟滚动 + 状态管理优化完成
    ├─ Day 54: 终端兼容性测试完成
    └─ Day 55: 最终验收，计划结束
```

---

## 六、风险评估与应对

| 风险 | 概率 | 影响 | 应对措施 |
|------|------|------|---------|
| 组件体系统一方案复杂度过高 | 中 | 高 | 预留简化方案：直接全部迁移到 ratatui Widget |
| 虚拟滚动实现困难 | 中 | 高 | 备选方案：分页加载（每次加载 50 条） |
| 状态管理重构引入回归 | 高 | 高 | 增量重构，每步均有完整测试覆盖 |
| 快捷键系统与现有 crossterm 事件冲突 | 中 | 中 | 详细的事件优先级文档，充分测试 |
| 开发周期超出预期 | 中 | 中 | 按 Phase 交付，每 Phase 可独立发布 |

---

## 七、资源与依赖

### 7.1 开发资源

- **架构负责人**：1 人（负责 A1、D2，评审关键 PR）
- **核心功能开发者**：1-2 人（负责 B1、B5、C1 等核心功能）
- **组件开发者**：1-2 人（负责组件修复、增强）
- **性能优化开发者**：1 人（负责 D1、D3）
- **QA/测试开发者**：1 人（负责 A4、D4，编写测试）

### 7.2 外部依赖

| 依赖 | 状态 | 说明 |
|------|------|------|
| ratatui 0.29+ | ✅ 已集成 | 无需变更 |
| crossterm 0.28+ | ✅ 已集成 | 无需变更 |
| syntect | ✅ 已集成 | 代码高亮依赖 |
| tokio | ✅ 已集成 | 异步运行时 |
| arboard | 📋 需添加 | 跨平台剪贴板（Phase C4） |
| fuse-rs / fuzzy-matcher | 📋 需添加 | 模糊搜索（Phase B1） |

### 7.3 关键文件索引

| 路径 | 用途 | 涉及任务 |
|------|------|---------|
| `rust/crates/yunxi-cli/src/tui/core/` | 核心框架 | A1, D2 |
| `rust/crates/yunxi-cli/src/tui/components/` | 组件库 | A2, A3, A4, B2-B5, C1-C5 |
| `rust/crates/yunxi-cli/src/tui/widgets/` | ratatui Widget | A1, B1-B5 |
| `rust/crates/yunxi-cli/src/tui/theme/` | 主题系统 | C2 |
| `rust/crates/yunxi-cli/src/tui/keymap/` | 快捷键系统 | C1 |
| `rust/crates/yunxi-cli/src/tui/session/` | 会话管理 | B5, C5 |
| `rust/crates/yunxi-cli/src/tui/clipboard/` | 剪贴板 | C4 |
| `rust/crates/yunxi-cli/src/tui/workspace/` | 工作区 | C5 |
| `rust/crates/yunxi-cli/src/tui/diff/` | 差异可视化 | C3 |
| `rust/crates/yunxi-cli/src/tui/syntax/` | 语法高亮 | B4 |
| `rust/crates/yunxi-cli/src/tui/app_ratatui.rs` | 主应用渲染 | B5, D1, D3 |

---

## 八、附录

### 附录 A：功能对齐检查表

| 功能 | opencode 有 | 当前状态 | Phase 目标 | 验收方式 |
|------|-----------|---------|-----------|---------|
| 命令面板 (Ctrl+P) | ✅ | ⚠️ 骨架 | ✅ 完整实现 | B1 |
| 侧边栏导航 | ✅ | ⚠️ 未集成 | ✅ 完整集成 | B5 |
| 快捷键系统 | ✅ | ⚠️ 基础 | ✅ 可自定义 | C1 |
| 多主题支持 | ✅ | ⚠️ 2 主题 | ✅ 可切换+编辑 | C2 |
| 插件系统 UI | ✅ | ⚠️ 示例 | ⚠️ 保持示例 | — |
| 文件差异可视化 | ✅ | ⚠️ 解析器 | ✅ 彩色渲染 | C3 |
| 工具调用折叠 | ✅ | ⚠️ 基础面板 | ✅ 折叠/展开 | B2 |
| 会话管理 | ✅ | ⚠️ SessionPicker | ✅ 增强体验 | B5 |
| 权限确认 | ✅ | ⚠️ 基础覆盖层 | ⚠️ 保持现状 | — |
| 代码高亮 | ✅ | ⚠️ 部分 | ✅ 流式集成 | B4 |
| 思考过程展示 | ✅ | ❌ 被注释 | ✅ 可视化 | B3 |
| 进度指示器 | ✅ | ⚠️ 部分 | ✅ 修复启用 | A2 |
| 错误处理 | ✅ | ⚠️ ErrorDialog | ✅ 修复启用 | A3 |
| 多选复制 | ✅ | ❌ 未实现 | ✅ 选择+复制 | C4 |
| 工作区管理 | ✅ | ❌ 未实现 | ✅ 文件树 | C5 |
| 对话复制 | ✅ | ❌ 未实现 | ✅ 单条/全部复制 | C4 |

### 附录 B：参考资源

1. **opencode TUI 设计参考**：`docs/superpowers/specs/2026-06-02-opencode-tui-redesign-design.md`
2. **ratatui 官方文档**：https://ratatui.rs/
3. **crossterm 事件系统**：https://docs.rs/crossterm/
4. **TUI 开发日志**：`rust/TUI-DEVELOPMENT-LOG.md`
5. **现有增强计划**：`rust/TUI-ENHANCEMENT-PLAN.md`（部分过时）

---

**计划制定人**：YunXi Agent  
**审阅人**：待指定  
**批准人**：待指定  

**变更记录**：

| 日期 | 版本 | 变更内容 | 作者 |
|------|------|---------|------|
| 2026-06-04 | 1.0 | 初始计划制定 | YunXi Agent |
