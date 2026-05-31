# 云熙 TUI 改造 + Tauri 桌面应用设计方案

**创建日期**: 2026-05-29
**状态**: Draft
**版本**: v0.1

## 1. 背景与动机

### 1.1 现状

云熙智能体当前 TUI 基于自研 ANSI 帧缓冲系统（`yunxi-cli/src/tui/frame.rs`），不依赖任何 TUI 框架。该系统存在以下局限：

- **渲染能力薄弱**：Markdown 渲染仅有基础支持，无 Diff 渲染、无动效
- **维护成本高**：75+ 个文件的手工 ANSI 渲染逻辑，缺乏框架抽象
- **扩展性差**：新增 UI 组件需要直接操作 ANSI 转义序列
- **无图形界面**：所有交互限于终端，无法提供可视化图表、文档预览等能力

### 1.2 参照标的

BCIP Agent（`/Users/xujian/projects/BCIP`）是同源的终端 AI 助手，其 TUI 基于 ratatui + crossterm（深度定制 fork），具备：

- 完整的 Markdown 渲染管线（表格、代码高亮、超链接）
- Diff 差分渲染（主题感知背景色、Hunk 语法高亮）
- Shimmer 动画 + 9 种 Spinner
- CIE76 感知色差匹配 + 终端探测
- Inline viewport 模式（对话写入终端滚动缓冲区）
- 事件驱动的异步架构（`tokio::select!` 五路事件合并）
- ~130 个 crate 的细粒度模块拆分

### 1.3 目标

1. 将 TUI 渲染引擎从自研 ANSI 帧缓冲迁移到 ratatui 框架
2. 借鉴 BCIP 的设计模式（非直接复制），提升渲染能力
3. 构建基于 Tauri 2.x 的跨平台桌面应用

## 2. 方案选择

### 2.1 三种考虑方案

| 方案 | 描述 | 评估 |
|------|------|------|
| A: 渐进式三步走 | Phase 1: ratatui 迁移 → Phase 2: 体验打磨 → Phase 3: Tauri 桌面 | **选定** |
| B: 双轨并行 | TUI 最小迁移 + Tauri 原型并行推进 | 资源分散，返工风险高 |
| C: 统一抽象层 | 先设计共享 UI 接口再实现两端 | 过度设计风险 |

### 2.2 选择理由

方案 A 每阶段产出可独立交付验证，风险可控。Phase 1 建立的 ratatui 组件在 Phase 3 的 Tauri 前端可复用其设计思想（组件化、状态管理）。runtime 核心层全程不变。

## 3. 整体架构

```
yunxi-cli/
├── tui/                        # ratatui TUI 层 (Phase 1-2)
│   ├── app.rs                  #   应用状态 & 事件循环
│   ├── terminal.rs             #   终端管理 (借鉴 BCIP custom_terminal)
│   ├── theme/                  #   主题 & 色彩感知 (借鉴 BCIP color/palette)
│   ├── markdown/               #   Markdown 渲染管线 (借鉴 BCIP)
│   ├── diff.rs                 #   Diff 渲染 (借鉴 BCIP)
│   ├── motion.rs               #   动效系统 (Shimmer/Spinner，借鉴 BCIP)
│   ├── streaming/              #   流式输出 (借鉴 BCIP)
│   ├── layout.rs               #   布局计算
│   ├── components/             #   可复用 TUI 组件
│   │   ├── chat_view.rs
│   │   ├── input_bar.rs
│   │   ├── markdown_view.rs
│   │   ├── diff_view.rs
│   │   ├── spinner.rs
│   │   └── ...
│   └── patent/                 #   专利专屏 (保留并 Widget 化)
│
├── desktop/                    # Tauri 桌面层 (Phase 3, 新增)
│   ├── main.rs                 #   Tauri 应用启动
│   ├── commands/               #   Tauri commands (Rust → Webview)
│   └── frontend/               #   前端 (Svelte 推荐)
│
├── live_cli.rs                 # 非 TUI REPL (保留)
└── main.rs                     # 入口统一路由

Core (不变):
  runtime/  llm/  tools/  workflow/  patent-domain/  memory/  knowledge/ ...
```

### 架构原则

- **ratatui 替代自研 Frame**：删除 `frame.rs`、`ansi.rs`，由 ratatui `Terminal`/`Frame`/`Buffer` 接管
- **组件 Widget 化**：所有 UI 单元实现 ratatui `Widget` trait
- **runtime 层零改动**：TUI 和 Tauri 都是 runtime 的消费者
- **Crossterm 保留**：作为 ratatui 的后端（终端原始模式 + 事件输入）

## 4. Phase 1: ratatui 迁移 (Week 1-4)

### 4.1 迁移策略

渐进替换，每周末可运行、可演示。旧的 `frame.rs` 在 Week 4 之前保留作为回退参考。

### 4.2 删除清单

| 文件 | 原因 |
|------|------|
| `tui/frame.rs` | 自研 ANSI 帧缓冲 → ratatui::Frame |
| `tui/ansi.rs` | ANSI 着色工具 → ratatui::style |
| `tui/ui_palette.rs` | 合并入新 theme 模块 |
| `tui/theme.rs` | 重写为 ratatui 主题系统 |
| `tui/banner.rs` | 改为 Widget 实现 |
| `tui/overlays.rs` | 分散到各组件自身 |
| `tui/pager.rs` | ratatui Paragraph 滚动替代 |
| `patent/render.rs` | Widget 重写 |
| `patent/layout.rs` | ratatui Layout 替代 |

### 4.3 新建清单

| 文件 | 内容 |
|------|------|
| `tui/terminal.rs` | 终端管理，借鉴 BCIP custom_terminal.rs 的 inline viewport 模式 |
| `tui/theme/mod.rs` | ratatui 主题系统（Color + Palette + 变体） |
| `tui/components/` (重写) | 全部改为 Widget trait 实现 |
| `tui/markdown/mod.rs` | Markdown → ratatui 渲染管线 |
| `tui/app.rs` (重写) | 事件驱动循环重构 |
| `tui/runner.rs` (重写) | REPL 事件循环 |
| `patent/` (重写) | 专利专屏 Widget 化 |

### 4.4 组件 Widget 树

```
App (StatefulWidget)
├── TitleBar          →  impl Widget for TitleBar
├── MainLayout        →  ratatui::layout::Layout
│   ├── ChatView      →  impl StatefulWidget for ChatView
│   └── ToolPanel     →  impl StatefulWidget for ToolPanel
├── InputBar          →  impl Widget for InputBar
│   └── SlashComplete →  impl Widget (PopUp)
├── StatusBar         →  impl Widget for StatusBar
├── HelpOverlay       →  impl Widget (Clear)
├── PermissionOverlay →  impl Widget
└── PatentScreen      →  impl StatefulWidget for PatentScreen
```

### 4.5 借鉴 BCIP 的 5 个设计模式

1. **事件驱动循环**（app.rs: run()）— `tokio::select!` 五路事件合并，mpsc::channel 连接 runtime → TUI
2. **HistoryCell 特质**（chatwidget.rs）— 抽象 Message/Diff/ToolCall 为统一的 Cell 类型
3. **Inline Viewport**（custom_terminal.rs）— 输出写入终端滚动缓冲区，非 Alt-Screen
4. **色彩感知**（color.rs + terminal_palette.rs）— CIE76 色差匹配 + 三档降级
5. **流式输出控制**（streaming/）— Markdown 流收集 → 自适应 drain → 打字机效果

### 4.6 Week-by-Week

| 周 | 任务 | 产出 |
|----|------|------|
| Week 1 | ratatui 骨架：Terminal 封装、事件循环、空渲染循环 | 项目可编译运行 |
| Week 2 | 迁移简单组件：TitleBar、StatusBar、InputBar | 基础 UI 可见 |
| Week 3 | 迁移有状态组件：ChatView（含 Markdown 管线）、ToolPanel | 对话可用 |
| Week 4 | 迁移专利专屏 + 覆盖层 + 集成回归测试 | 功能完整，回归通过 |

## 5. Phase 2: 体验打磨 (Week 5-8)

### 5.1 Markdown 渲染管线

```
Markdown 文本 → fence 展开 → pulldown-cmark 解析
→ syntect 语法高亮 → 宽度感知换行 → 表格自适应
→ OSC 8 超链接 → Vec<Line> → ChatView
```

### 5.2 Diff 差分渲染

- 新增行/删除行背景色区分
- 暗色终端：绿 `#213A2B` / 红 `#4A221D`
- 亮色终端：GitHub 风格（`#dafbe1` / `#ffebe9`）
- Hunk 内语法高亮状态保持
- 三档颜色降级（TrueColor → 256 → 16）

**专利场景**：权利要求修改前后对比、审查意见答复 diff

### 5.3 动效系统

- **Shimmer**：正弦波扫光效果（活动指示器）
- **Spinner**：5 种样式 × 36 帧（Braille/Dots/Line/Arrow/Moon）
- **reduced-motion**：静态指示器，适配 `$NO_MOTION` 环境变量

### 5.4 色彩感知系统

```
终端能力探测 ($COLORTERM / $TERM)
  → TrueColor → 256色 (降级)
默认色探测 (ANSI escape)
  → 暗色/亮色自动判断
主题匹配 (CIE76 感知色差, Lab 色彩空间)
  → 自动选择最佳 palette
用户主题 (--theme CLI 参数)
  → Dark / Light / Monokai / Nord / HighContrast
```

### 5.5 专利专屏增强

- 权利要求对比视图：左右分屏 diff（独立权利要求 vs 对比文件）
- 审查意见标注：OA 关键论点高亮 + 法律条款引用
- 导航增强：Ctrl+1~6 快捷键 + 鼠标点击（ratatui 鼠标支持）
- 证据面板：ASCII art 表格渲染

### 5.6 Week-by-Week

| 周 | 任务 | 产出 |
|----|------|------|
| Week 5 | Markdown 渲染管线 | 表格/代码高亮/超链接 |
| Week 6 | Diff 渲染 + 动效 | Diff 视图 + Shimmer/Spinner |
| Week 7 | 色彩感知系统 | 终端探测 + CIE76 + palette |
| Week 8 | 专利专屏增强 + 集成 | 精致 TUI v2.0 |

## 6. Phase 3: Tauri 桌面应用 (Week 9-14)

### 6.1 为什么选 Tauri

| 对比 | Tauri 2.x | Electron | Flutter | Qt |
|------|-----------|----------|---------|-----|
| 后端语言 | Rust (与项目同构) | Node.js | Dart | C++ |
| 安装包体积 | <10MB | 100MB+ | 20MB+ | 30MB+ |
| IPC 开销 | 零拷贝 (同进程调用) | 跨进程序列化 | 跨语言 FFI | C FFI |
| 移动端 | 2.x 支持 iOS/Android | 无原生支持 | 支持 | 有限 |
| 安全性 | CSP + 权限模型 | 需自行配置 | 沙箱 | 无 |

### 6.2 架构

```
yunxi-cli/src/desktop/ (模块，非独立 crate)
├── main.rs                 #   Tauri 应用启动（独立 binary target）
├── commands/               #   Tauri commands (Rust → Webview)
│   ├── mod.rs
│   ├── chat.rs             #     chat_send(msg) → runtime::Session
│   ├── patent.rs           #     patent_draft(doc) → tools::patent_drafting
│   ├── search.rs           #     search_kb(query) → knowledge
│   └── events.rs           #     stream_events() → event_bus
└── frontend/               #   前端 (Svelte 推荐)
    └── invoke("command_name", args)  ←→  Rust backend
```

### 6.3 前端选型

**推荐 Svelte**：编译后体积最小（&lt;5KB 运行时）、响应式简单、适合以表单/文档展示为主的应用。Phase 3 启动时最终确认。

备选：React（生态丰富但体积大）、Leptos（纯 Rust/WASM 但调试困难）。

### 6.4 核心功能

| 模块 | 功能 |
|------|------|
| 专利工作流 | 技术交底书导入、5 阶段撰写向导、权利要求树形编辑器、说明书分段编辑、一键导出 .docx |
| 检索与分析 | 多源专利检索、对比文件可视化、相似度热力图、审查意见标注、报告生成（含图表） |
| AI 助手 | 对话式交互、多模型切换、Token 用量仪表盘、会话历史管理、知识库搜索 |

### 6.5 模块结构

`desktop/` 作为 `yunxi-cli` 内的模块（非独立 crate），直接复用现有依赖：

```rust
// yunxi-cli/src/desktop/main.rs  — 独立的 Tauri 二进制入口
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// yunxi-cli/src/desktop/commands/ — Tauri commands 复用 runtime bridge
```

`yunxi-cli/Cargo.toml` 仅新增 `tauri` 和 `tauri-build` 两个依赖（见第 7 节），其余共享 crate 的依赖无需重复声明。

### 6.6 Week-by-Week

| 周 | 任务 | 产出 |
|----|------|------|
| Week 9-10 | Tauri 骨架 + 前端搭建 | 窗口可启动、IPC 通信验证 |
| Week 11-12 | 专利工作流 GUI | 撰写向导、权利要求编辑器 |
| Week 13 | 检索与分析面板 | 可视化对比、报告导出 |
| Week 14 | 打包发布 + CI/CD | dmg/deb/msi 安装包 |

## 7. 依赖变更汇总

```toml
# yunxi-cli/Cargo.toml

# 保留
pulldown-cmark = "0.13"    # Phase 2 Markdown 管线继续使用
syntect = "5"              # 语法高亮继续使用

# 升级
~ ratatui: optional → required (移除 feature gate "tui")
~ crossterm: 0.28 (不变)

# 新增 (Phase 3)
+ tauri = "2"
+ tauri-build = "2"
```

> 注：`desktop/` 作为 `yunxi-cli` 内的模块（非独立 crate），与 TUI 共享 runtime bridge、配置管理等模块。

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| ratatui 学习曲线 | Phase 1 延期 | Week 1 搭建最小可运行骨架，提前验证关键路径 |
| CJK 字符宽度 | 专利中文内容显示错位 | ratatui 已支持 Unicode 宽度，Week 3 验证 |
| 自研 Frame 功能未覆盖 | 功能退化 | Week 4 完整的回归测试 + feature parity 检查 |
| Tauri 生态不熟悉 | Phase 3 推进缓慢 | Week 9 仅搭建骨架，使用 Tauri 官方模板 |
| 专利专屏迁移复杂度 | 核心功能受损 | 保留旧代码为参考，按功能逐块迁移 |

## 9. 成功标准

- **Phase 1 完成**：`cargo run -- tui` 可进入 ratatui TUI，现有功能不退化
- **Phase 2 完成**：Markdown/Diff/Shimmer 全部可用，色彩感知正常工作，回归测试通过
- **Phase 3 完成**：`yunxi-desktop` 在 macOS 上能启动、IPC 通信正常、可打包分发
- **全流程**：`cargo fmt` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` 全部通过
