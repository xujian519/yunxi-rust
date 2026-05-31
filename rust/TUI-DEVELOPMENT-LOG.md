# YunXi TUI 开发进度日志

按时间记录 `yunxi-cli` 全屏 TUI 与相关 CLI 的演进，便于接续开发与评审。

---

## 2026-05-30 — 专利专屏 TUI 退役

**状态**：🟢 已删除 `tui/patent/`、`--patent` / `--profile patent` / `YUNXI_UI_MODE=patent`；终端 `yunxi` 仅保留通用全屏布局。

| 项 | 说明 |
|----|------|
| 替代产品 | `yunxi-desktop`（`rust/crates/yunxi-cli` + `frontend/`） |
| CLI | `--patent` 返回友好错误并指向桌面构建命令 |
| 会话 | `merge_save_session` 仍保留 JSON 中已有 `patentCase` 字段 |
| 项目 `/init` | 检测到 `YUNXI.md` 含 `patentCase` 时提示使用桌面端 |

历史 Phase 7 专屏实现见本文件下方归档条目，代码已不再存在于仓库。

---

## 2026-05-28 — v1 基线达成

**目标**：默认 `yunxi` 进入可用全屏 TUI，能力与行式 REPL 对齐。

| 类别 | 内容 |
|------|------|
| 架构 | `main.rs` ~154 行；`live_cli.rs`；`tui/` crossterm 帧缓冲（无 ratatui） |
| Phase 0–6 | 状态栏、流式 Markdown、工具面板、斜杠/分页/会话选择、主题与 spinner |
| 测试 | `cargo test -p yunxi-cli` 全绿 |

---

## 2026-05-28 — v1.1 交互增强

| 项 | 说明 |
|----|------|
| 工具面板 | `format_tool_panel_detail` 语法高亮；Done 时格式化写入 |
| 流式进度 | `TurnObserver::on_usage` + 字符估算兜底；状态栏 `[████░░░░]` |
| Phase 0 收尾 | `oauth_flow.rs`、`resume_session.rs`、`cli_tests.rs` |
| 配置 | `showBanner`；REPL/TUI 共用 |
| 斜杠参数 Tab | `slash_complete_shared`：`/model`、`/permissions`、`/session`、`/config`、`/export` |
| 鼠标 | 滚轮滚动；工具面板左键折叠；轮次中可滚对话 |
| 贴底跟随 | 流式时仅在已到底部自动滚到最新 |
| Ctrl+F | 预填 `/search` |
| 文档 | `TUI-ENHANCEMENT-PLAN.md` 标 v1 完成；Canvas `yunxi-tui-guide.canvas.tsx` |

---

## 2026-05-28 — v1.2 布局微调（当前）

| 项 | 说明 |
|----|------|
| 输入框 | 默认最小高度由 **5 行 → 4 行**（`DEFAULT_MIN_ROWS`） |
| 优先级 | **Phase 7 专利工作流专屏** 提升为最高优先级（见下） |

---

## 2026-05-28 — Phase 7 v0.9：国知局版式导出（当前）

**状态**：🟢 可运行 v0；持续深化中。

**入口**（推荐）：

- **`yunxi --patent`**（主入口）
- `yunxi --profile patent`（兼容）
- 配置 `uiMode` / `ui_mode` / `profile`: `"patent"`
- 环境变量 `YUNXI_UI_MODE=patent`

**已实现（v0）**：

| 模块 | 路径 |
|------|------|
| 模式解析 | `tui/ui_mode.rs` |
| 三栏布局 | `tui/patent/layout.rs`、`render.rs` |
| 导航与工作区 | `tui/patent/workspace.rs`（6 视图：权利要求 / 对比 / 审查意见 / 检索 / 草稿 / 对话） |
| 工具结果灌入 | `ingest.rs`、`compare_format.rs`（PatentCompare JSON→矩阵） |
| 案件元数据 | `patent/config.rs` ← `patentCase` |
| 专屏帮助 | `patent/help_overlay.rs`（F1） |
| 快捷键 | `1`–`6`、`F3`/`F2`/`F1`、主区 `j/k/g/G` |

**布局**（三栏 + 底栏，与 Canvas 示意一致）：

1. **左栏** — 案件导航与元数据占位
2. **中栏** — 主工作区（对照表、形式检查、检索列表、助手摘要等）
3. **右栏** — 证据与工具输出（复用 `ToolPanel`）
4. **底栏** — 4 行输入 + 状态栏

**v0.3 专利 `/init` 工作流**：

| 能力 | 说明 |
|------|------|
| 文件夹扫描 | 识别权利要求/审查意见/对比文件/说明书等 |
| 意图推断 | 答复审查意见、撰写申请、对比分析等 |
| `YUNXI.md` | frontmatter `patentCase` + 可编辑「工作约定」；`/init` 只刷新 `<!-- yunxi:patent-scan -->` 块 |
| 专屏 `/init` | 扫描 → 写 YUNXI.md → 灌入专屏；`yunxi init` 在专利目录自动走专利模板 |

**推荐流程**：`cd 案件文件夹` → `yunxi --patent` → `/init` → 编辑 `YUNXI.md` → 开始对话。

**v0.4 办公格式与提示词**：

| 能力 | 说明 |
|------|------|
| `/init` 办公清单 | PDF/Word/图片等单独列表 + 建议调用 `PdfParse`/`DocxParse` 等 |
| 启动引导 | 无 `YUNXI.md` 时提示先 `/init` |
| 系统提示词 | 注入办公工具目录 + `YUNXI.md`「工作约定」章节 |

**v0.2 新增**：

| 能力 | 说明 |
|------|------|
| 专屏斜杠 | `/help` `/case` `/case set` `/import` `/view` |
| 会话 `patentCase` | 写入 `.yunxi/sessions/*.json`，退出/轮次结束持久化 |
| 历史回放 | 切换会话或加载时从工具结果重建各面板 |
| 案号推断 | 用户消息中的 `CN…` 申请号自动识别 |

**v0.5（2026-05-28）**：

| 能力 | 说明 |
|------|------|
| 状态栏案号 | `patent_case_hint`：案号 · 阶段（专利专屏底栏） |
| `/extract` 灌入 | 抽取成功后按文件名写入权利要求/审查意见/对比等主视图 |
| 工具灌入 | `OaParse`、`VisionOcr`/`PdfParse`/`DocxParse` 等结果自动进面板 |
| oMLX OCR | `gemma-4-e2b-it-4bit` + 与 BGE-M3 共用 API Key |

**v0.6（2026-05-28）**：

| 能力 | 说明 |
|------|------|
| 扫描 PDF | `PdfParse` + `scanned_ocr`：`pdftoppm` 分页 → oMLX Vision（回退 Tesseract） |
| `/extract` | 检测文本层过少自动走分页 OCR，报告标注「扫描OCR」 |

**v0.7（2026-05-28）**：

| 能力 | 说明 |
|------|------|
| `/ocr <路径>` | 单文件抽取 + 灌入（PDF 含扫描分页 OCR） |
| `/panel` | 分页器查看当前主视图 |
| `/materials` | 材料清单与工具建议 |
| `/extract [过滤]` | 支持按文件名子串过滤或多文件 |
| `/import` | PDF/图片走 OCR 而非仅 read_file |
| Tab 补全 | 专利专屏斜杠命令与路径 |

**v0.8（2026-05-28）**：

| 能力 | 说明 |
|------|------|
| `/preview <路径>` | 单文件抽取全文预览（分页器）；**不灌入主视图**；扫描 PDF 自动分页 OCR |
| 纯文本 | `.md` / `.txt` 直接读入预览，不走工具链 |

**v0.9（2026-05-28）**：

| 能力 | 说明 |
|------|------|
| `/export` | 国知局版式导出（Markdown + DOCX）；CNIPA 意见陈述书模版 |
| `/export docx` | 经由 Python `docx_export.py` 生成 Word 文件（需 python-docx） |
| `/export md` | 直接导出结构化 Markdown（零外部依赖） |
| 零警告 | 全项目 `cargo check` 零警告 |

**v0.10（2026-05-28）**：

| 能力 | 说明 |
|------|------|
| CNIPA 增强 | `cnipa.rs` 重写：重试(3次指数退避)、结构化解析(EPUB_HITS_JSON)、详情/批量下载 |
| 高被引检索 | `HighCitationPatents` 从存根实现 → 调用 CNIPA 检索并按引用排序 |
| 批量下载 | `BatchPatentDownload` 从存根实现 → 调用 CNIPA PDF 下载，逐个下载并汇总 |
| /flow 增强 | 新增 `/flow clear` 清除挂起流程子命令；改进帮助文案 |
| Phase 5 验证 | 记忆系统 (Hebbian + 4层 Tier + Store + tools) 集成验证通过 |

**待办**：ratatui 迁移。

**依赖**：`brew install poppler`（pdftoppm）、oMLX :8009 + API Key。

**不在本期**：ratatui 迁移、国知局版式导出、独立法条浏览器窗口。

---

## 2026-05-30 — ratatui 迁移 Phase 1 + 浅色背景适配

### ratatui 迁移

| 项 | 说明 |
|----|------|
| 渲染引擎 | 从自研 ANSI 帧缓冲迁移到 ratatui Widget 体系 |
| 事件循环 | `terminal.draw(|frame| app.render_frame(frame))` |
| 组件 | `ChatViewWidget`、`InputBarWidget`、`TitleBar`、`StatusBarWidget`、`ToolPanelWidget`、`HelpOverlay`、`PatentScreenWidget` |
| 测试 | `cargo test -p yunxi-cli` 224 项全绿 |

### 浅色背景自适应配色

**问题**：所有 ratatui widget 使用硬编码 256 色值（`183`浅紫、`213`浅粉、`245`浅灰），白色背景下对比度极低，文字模糊不可读。

**修复**：在 `ui_palette.rs` 中新增自适应颜色函数，根据 `terminal_light_background()` 自动返回深色/浅色适配值。

| 用途 | 函数 | 深底色值 | 浅底色值 |
|------|------|---------|---------|
| 品牌/主色 | `user_role_color()` | `183` 浅紫 | `55` 深紫 |
| 强调色 | `accent()` | `213` 浅粉 | `162` 深洋红 |
| 次要文字 | `dim_color()` | `245` 浅灰 | `243` 中灰 |
| 选中/高亮 | `highlight()` | `214` 橙 | `166` 深橙 |
| 正文内容 | `content_color()` | `252` 浅白 | `235` 深灰 |
| 助手标签 | `assistant_role_color()` | `213` 浅粉 | `162` 深洋红 |
| 系统标签 | `system_role_color()` | `246` 浅灰 | `242` 中灰 |

**修改文件**：

| 文件 | 变更 |
|------|------|
| `ui_palette.rs` | 新增 `brand()`、`accent()`、`dim_color()`、`highlight()`、`user_role_color()`、`assistant_role_color()`、`system_role_color()`、`content_color()` 自适应函数 |
| `chat_view_ratatui.rs` | 角色、正文、spinner 颜色改用自适应函数 |
| `title_bar.rs` | 品牌名、强调色、暗淡色改用自适应函数 |
| `status_bar_ratatui.rs` | 全部 segment 颜色改用自适应函数 |
| `tool_panel_ratatui.rs` | 工具名、详情、边框颜色改用自适应函数 |
| `help_overlay_ratatui.rs` | 边框、快捷键、描述颜色改用自适应函数 |
| `patent_screen_ratatui.rs` | 导航、案件信息、对比表、证据面板全部颜色改用自适应函数 |
| `app_ratatui.rs` | 弹窗颜色改用自适应函数 |

### 斜杠命令补全菜单

**问题**：`InputBarWidget` 未渲染补全候选列表，只显示 `(N 补全)` 数字。

**修复**：`input_bar_ratatui.rs` 完整重写，新增 `slash_completion` 字段，支持：
- 最多 5 行候选列表渲染
- 选中项高亮（白字深蓝底）
- 未选中项暗淡显示
- 底部提示行（Tab 应用、↑↓ 选择、当前命令描述）
- 空输入占位文字

**修改文件**：

| 文件 | 变更 |
|------|------|
| `input_bar_ratatui.rs` | 新增 `slash_completion` 字段，完整补全菜单渲染 |
| `app_ratatui.rs` | 传入 `slash_completion` 引用 |

---

## 2026-05-30 — 配置加载修复（TUI + 桌面共用）

| 问题 | 修复 |
|------|------|
| `~/.yunxi/settings.local.json` 未参与合并 | `ConfigLoader::discover` 增加用户级 local |
| 从 `rust/` 启动找不到项目 `.yunxi/` | `resolve_project_cwd` 向上查找 |
| LLM 密钥未注入环境 | `apply_merged_config_env()` 于 CLI 启动 |

**验证**：在 `rust/` 目录执行 `yunxi doctor` 应显示 `DEEPSEEK_API_KEY 已设置`。

桌面客户端进度见 [`crates/yunxi-cli/DESKTOP-DEVELOPMENT-LOG.md`](crates/yunxi-cli/DESKTOP-DEVELOPMENT-LOG.md)。

---

## 验证命令

```bash
cd rust && cargo test -p yunxi-cli
cargo run -p yunxi-cli
cargo run -p yunxi-cli -- --patent
```
