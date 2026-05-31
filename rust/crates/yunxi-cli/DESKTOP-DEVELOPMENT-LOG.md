# 云熙桌面前端开发进度日志

按时间记录 `yunxi-cli` Tauri 2 + React 桌面客户端的演进。设计规范见 [`docs/superpowers/specs/2026-05-30-yunxi-desktop-frontend-design.md`](../../../docs/superpowers/specs/2026-05-30-yunxi-desktop-frontend-design.md)。

---

## 2026-05-31 — 默认模型 DeepSeek + TUI 斜杠补全

**状态**：🟢 完成。

| 项 | 说明 |
|----|------|
| 默认模型 | `.yunxi/settings.json` → `deepseek-v4-pro`；`normalize_startup_model`：`auto`/无密钥 → DeepSeek |
| 斜杠 | 仅输入 `/` 回车 → `/help`；有补全菜单时回车先应用选中项（如 `/models`） |

---

## 2026-05-30（续）— 材料预览 / Slash→AI / 快捷键合并 / 最近命令

**状态**：🟢 完成。

| 项 | 说明 |
|----|------|
| 材料导入 | `list_project_materials` 预览对话框，确认后导入；Explorer / 命令面板走 `startImportProjectMaterials` |
| Slash | `executeSlashCommand` 默认同步到 AI 对话 + 输出面板 |
| 快捷键 | `mergeShortcutDefaults`：旧 settings 自动补全 `command-palette` 等新增项 |
| 命令面板 | `localStorage` 记录最近 8 条，空搜索时置顶 |
| 编辑器字体 | `--editor-font-family` 默认写入根样式；权利要求/对比/检索视图消费 |

---

## 2026-05-30（续）— 命令面板增强（Slash / 案件 / 项目）

**状态**：🟢 完成。

| 项 | 说明 |
|----|------|
| Slash | `/help` `/status` `/cost`；`/search` `/analyze` 二级输入；结果写入底栏输出 |
| 案件 | 新建案件、刷新列表、导入当前案件材料 |
| 工作区 | 命令面板列出已扫描项目（最多 12 个）并跳转 |

`runSlashCommand` 抽至 `utils/slashCommandRunner.ts`。

---

## 2026-05-30（续）— 命令面板 + 活动栏入口

**状态**：🟢 完成。

| 项 | 说明 |
|----|------|
| 命令面板 | `CommandPalette`：⇧⌘P 或活动栏搜索图标；视图/布局/工作区/设置等命令 |
| 活动栏 | 底部增加命令面板、设置、底栏入口 |

分发说明：本地/内测可不签名；他人 Mac 首次打开可能需在「隐私与安全性」中允许。

---

## 2026-05-30（续）— 快捷键 / 材料递归 / CI 桌面检查

**状态**：🟢 完成。

| 项 | 说明 |
|----|------|
| 快捷键 | `desktop.shortcuts` 持久化；`DesktopShortcutHandler` 绑定侧栏/AI/终端/主题/设置/搜索 |
| 材料导入 | `list/import_project_materials` 支持 `max_depth`（默认 2）；子目录相对路径作标题 |
| CI | `desktop` job：`npm run build` + `cargo check --features desktop` |

---

## 2026-05-30（续）— 外观应用 / Onboarding / PTY resize

**状态**：🟢 三项完成。

| 项 | 说明 |
|----|------|
| 外观 CSS | `applyDesktopAppearance` + `AppearanceBridge`：强调色、字号、密度、动画写入 CSS 变量 |
| 首次引导 | `#/onboarding` + `OnboardingGuard`；`save_llm_api_key` 写入 `env.{DEEPSEEK_API_KEY}` |
| PTY resize | `shell_session_resize`；底栏高度变化时更新 rows/cols |

---

## 2026-05-30（续）— 设置页与 settings.json 同步

**状态**：🟢 主要设置分类已读写 `.yunxi/settings.json`。

| 分类 | 持久化字段 |
|------|------------|
| 模型 | `model`、`apiKeys`、`desktop.model` |
| 通用 | `desktop.general`、`permissions.defaultMode` |
| 外观 | `desktop.appearance`（含 theme，与 ThemeProvider 联动） |
| 编辑器 | `desktop.editor` |
| 费用 | `desktop.cost`（预算）、`get_usage` 真实用量 |

`AppProvider` 新增 `yunxiSettings` / `persistYunxiSettings` / `updateDesktopSection`；状态栏预算来自 `desktop.cost.budgetUsd`。

---

## 2026-05-30（续）— PTY 终端 / 工作区监视 / 项目材料导入

**状态**：🟢 Rust IPC + 前端接线完成；`cargo check --features desktop` 与 `npm run build` 通过。

| 项 | IPC / 实现 |
|----|------------|
| 交互终端 | `shell_session_start` / `shell_session_write` / `shell_session_close`（`portable-pty`），事件 `yunxi://shell/{id}` |
| 扫描深度 | `scan_workspace_roots(paths, max_depth)`，默认 2、上限 5；Explorer 下拉 1–5 + `localStorage` |
| 目录监视 | `workspace_watch_start` / `workspace_watch_stop`（`notify`），debounce 700ms → `yunxi://workspace/changed` |
| 材料导入 | `list_project_materials` / `import_project_materials`（markitdown 脚本转换非文本） |

前端：底栏终端 Tauri 下走 PTY；工作区变更自动重扫；专利项目行「导入」按钮（当前案件匹配 `caseId` 时显示）。

---

## 2026-05-30（续）— 工作区扫描 / 原生选目录 / 真实 Shell

**状态**：🟢 三项深度集成完成。

| 项 | IPC / 实现 |
|----|------------|
| 原生选文件夹 | `pick_workspace_folder`（`rfd`） |
| 扫描 YUNXI.md | `scan_workspace_roots` → 解析 `patentCase` id/name |
| 真实终端 | `shell_exec(cwd, cmd)`，`sh -c`，60s 超时 |

前端：Explorer 展示扫描到的项目；终端在工作区目录执行命令；添加文件夹优先系统对话框。

---

## 2026-05-30（续）— VS Code 增强：底栏 / 多根工作区 / 标签拖拽

**状态**：🟢 三项按优先级落地。

| 优先级 | 项 | 说明 |
|--------|-----|------|
| P1 | 底部 Panel | `BottomPanel`：问题 / 输出 / 终端；`ResizablePanelVertical` 拖拽高度；活动栏与状态栏可唤起 |
| P2 | 多根工作区 | `workspaceFolders` + `localStorage`；Explorer「工作区」区段；主工作区来自 `get_workspace_info` |
| P3 | 标签拖拽 | `EditorTabBar` HTML5 拖拽排序 + `reorderEditorTabs` |

---

## 2026-05-30（续）— VS Code 式三栏布局

**状态**：🟢 活动栏 + 资源管理器 + 编辑器标签页 + 右侧 AI 面板。

| 项 | 说明 |
|----|------|
| `ActivityBar` | 资源管理器 / 对比 / 审查 / 检索（48px 竖条） |
| `ExplorerSidebar` | 案件树 + 会话列表（去掉六视图顶栏切换） |
| `EditorWorkbench` | 文档标签页 + 面包屑 + 中心视图 |
| `EditorTab` | `AppProvider` 管理打开标签与工具视图 |
| 右栏 | 仅 `RightPanel` AI 对话；中心 `ChatView` 不再作为主入口 |

---

## 2026-05-30（续）— Phase 3：Compare / Review / Draft 接案件文档

**状态**：🟢 中心四视图（检索、对比、审查、起草）在 Tauri 下均接 `~/.yunxi/cases/` 真实数据。

### 本批交付

| 项 | 说明 |
|----|------|
| 案件种子扩展 | `claims` / `drafts` / `review` / `description` 四类文档；既有 `case-1` 自动 `enrich_demo_case` 补全 |
| `lineDiff.ts` | 权利要求原始稿 vs 修改稿行级 diff |
| `reviewParse.ts` | 审查意见 JSON / 纯文本解析；答复写入 review 文档 |
| `CompareView` | 读取 `claims` + `drafts` 做左右对比 |
| `ReviewView` | 读取 `review` 文档；答复自动保存；「知识库参考」走 `knowledge_search` |
| `DraftView` | 编辑 `description` 文档，防抖保存到案件 |
| `/analyze` | 斜杠命令接 `knowledge_search` |

### Phase 3 进度

| 子项 | 状态 |
|------|------|
| Search / Chat / Compare / Review / Draft | ✅ Tauri 真实数据 |
| Claims 视图 | ✅ 沿用 `activeDocContent` |
| 设置页全量同步 | ✅ desktop 分区 + permissions + apiKeys |

---

## 2026-05-30（晚）— Phase 3 P0：检索视图 + 聊天结构化

**状态**：🟢 SearchView 接 `patent_search` IPC；RightPanel 支持权限弹窗、工具卡片、思考折叠。

### 本批交付

| 项 | 说明 |
|----|------|
| `utils/patentSearchParse.ts` | 解析 PatentSearch JSON；IPC 不可用时展示提示 |
| `SearchView.tsx` | Tauri 下回车/按钮触发真实检索；Mock 模式保留本地过滤 |
| `PermissionModal.tsx` | 响应 `permission_request` 流事件，调用 `permission_respond` |
| `ToolCallCard.tsx` | `tool_use` / `tool_result` 结构化展示（可折叠） |
| `ReasoningBlock.tsx` | `reasoning_delta` 折叠展示 |
| `AppProvider` | 流式事件改结构化字段；`/search` 斜杠输出摘要列表 |

### Phase 3 进度更新

| 子项 | 状态 |
|------|------|
| SearchView 真实检索 | ✅ |
| RightPanel 工具调用 + PermissionModal | ✅ |
| Review / Compare / Draft 真实数据 | ❌ 仍 Mock |
| Reasoning 折叠 | ✅ |

---

## 2026-05-30 — Phase 0–2 基线 + 全链路接线

**状态**：🟢 可日常开发；macOS `.app` 可打包；真实 LLM 对话已通。

### 里程碑

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 0 Tauri 验证 | ✅ | `yunxi-desktop` 加载 `dist/`，六视图可切换 |
| Phase 1 工程迁入 | ✅ | `frontend/` 源码、`api/` 层、设置页入口 |
| Phase 2 IPC Bridge | ✅ | 流式对话、会话/案件/设置/费用 IPC |
| Phase 3 视图接线 | ✅ | 六视图壳子 + 中心五视图（含 Claims）已接案件/IPC；仅 Mock 预览模式用假数据 |
| Phase 4 发布 | 🔄 | `npm run bundle:desktop` 可出 `.app` + `.dmg`；CI 未对齐 |

### 架构与目录

| 项 | 路径 |
|----|------|
| React 前端 | `frontend/src/` |
| Tauri 入口 | `src/desktop/main.rs` |
| IPC 命令 | `src/desktop/commands/` |
| 构建产物 | `dist/` → 嵌入 `yunxi-desktop` |
| 应用图标 | `icons/`（由 `frontend/public/app-icon.png` 生成） |
| 配置 | `tauri.conf.json` |

### 白屏问题（已修复）

| 根因 | 修复 |
|------|------|
| `set_current_dir(workspace)` 导致 Tauri 找不到 `dist/` | 只设 `YUNXI_WORKSPACE`，不 chdir |
| release 未启用 `custom-protocol` | `Cargo.toml` tauri feature 加 `custom-protocol` |
| Vite `target: esnext` WebView 静默失败 | 改为 `safari14`；`build.rs` 监听 `dist/**` |
| HTML script / crossorigin | script 放 body、去掉 crossorigin、启动占位文案 |

### 全链路 IPC（已实现）

| IPC | 说明 |
|-----|------|
| `chat_send` / `chat_cancel` | 流式对话 + 事件推送 |
| `session_*` | `.yunxi/sessions` 读写 |
| `case_*` | `~/.yunxi/cases/` 案件 CRUD |
| `get_settings` / `save_settings` | 合并配置读写 |
| `get_usage` | 会话 token 累计费用 |
| `patent_search` / `knowledge_search` | 检索桥接 |

### UI 完善（2026-05-30 晚）

| 项 | 说明 |
|----|------|
| 设置页布局 | `Layout contentMode="full"`，消除右侧空白 |
| 关于页 | 真实版本、`app-icon.png`、工作区路径、短期路线图 |
| 品牌图标 | `icons/icon.icns`；UI 统一 `./app-icon.png` |
| macOS 打包 | `bundle-desktop.sh`；`npm run bundle:desktop` + `open:desktop` |
| 注意 | Dock 图标需用 **`.app` 启动**，勿直接跑裸 `yunxi-desktop` 二进制 |

### 配置加载修复（桌面 + TUI 共用）

| 问题 | 修复 |
|------|------|
| `~/.yunxi/settings.local.json` 未被 `ConfigLoader` 加载 | 加入 discover 列表 |
| 从 `rust/` 子目录启动读不到项目 `.yunxi/` | `resolve_project_cwd` 向上查找 |
| 密钥未注入进程环境 | CLI 启动时 `apply_merged_config_env()` |

**相关文件**：`runtime/src/config.rs`、`yunxi-cli/src/lib.rs`、`session_mgr.rs`

### 打包命令（从仓库 `rust/` 目录）

```bash
# 推荐：一键打包并打开
./crates/yunxi-cli/bundle-desktop.sh

# 或分步
cd crates/yunxi-cli/frontend
npm run bundle:desktop
npm run open:desktop
# 等价绝对路径：
# open /Users/xujian/projects/YunXi/rust/target/release/bundle/macos/云熙智能体.app
```

**常见路径错误**：已在 `rust/` 时勿再 `cd rust/crates/...`；从 `frontend/` 到 `target/` 需 `../../../target`（三级）。

### 仅编译桌面二进制（开发调试）

```bash
cd rust/crates/yunxi-cli/frontend && npm run build:desktop
cd ../../.. && cargo build --release --features desktop -p yunxi-cli --bin yunxi-desktop
./target/release/yunxi-desktop
```

须看到 **`Compiling yunxi-cli`**，不能只有 `Finished 0.xx s`。

---

## 短期计划（约 2–4 周）

| 优先级 | 任务 | 状态 |
|--------|------|------|
| P0 | 中心视图真实数据（Review / Compare / Draft） | ✅ |
| P0 | PermissionModal、ToolCallCard、Reasoning 折叠 | ✅ |
| P0 | SearchView 接 patent_search | ✅ |
| P1 | 设置页与后端同步（模型/外观/费用） | ✅ |
| P1 | Onboarding 替代 Login Mock | ✅ `#/onboarding` + `save_llm_api_key` |
| P2 | macOS 签名 + CI 自动打包 | ⏭️ 不做签名；CI 仅 `desktop` check |
| P2 | 更新设计文档 Phase 验收勾选 | 进行中 |

---

## 验证命令

```bash
# 前端
cd rust/crates/yunxi-cli/frontend && npm run build:desktop

# Rust 桌面 feature
cd rust && cargo build --release --features desktop -p yunxi-cli --bin yunxi-desktop

# TUI（与桌面共用配置，用 yunxi 而非 yunxi-desktop）
cd rust && cargo build -p yunxi-cli --bin yunxi
./target/release/yunxi
yunxi doctor
```

---

## 关联文档

- [桌面前端设计规范](../../../docs/superpowers/specs/2026-05-30-yunxi-desktop-frontend-design.md)
- [TUI 开发日志](../../TUI-DEVELOPMENT-LOG.md)
- [本机使用指南](../../../LOCAL_SETUP.md)
