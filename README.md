<p align="center">
  <img src="assets/logo.svg" alt="云熙智能体 Logo" width="200" />
</p>

<h1 align="center">云熙智能体</h1>

<p align="center">
  <strong>YunXi Agent — 专业专利智能体</strong>
</p>

<p align="center">
  基于 Rust 构建的高性能专利智能处理引擎，为专利检索、分析与生成提供专业 AI 能力。
</p>

---

## 项目简介

云熙智能体（YunXi Agent）是一款面向专利领域的专业智能体工具链。项目采用 Rust 语言从零构建，具备高性能、内存安全和原生工具执行等特性，旨在为专利工作流提供全链路 AI 能力支撑。

### 核心定位

- **专利检索** — 智能化的专利文献搜索与筛选
- **专利分析** — 深度解析专利文本、权利要求和引用关系
- **专利生成** — 辅助生成专利文档、说明书和权利要求书
- **工作流集成** — 无缝对接专利审查、申请和管理流程

---

## 技术架构

项目采用 Rust workspace 多 crate 架构：

```text
rust/
├── Cargo.toml              # Workspace 根配置
├── Cargo.lock
└── crates/
    ├── adapters/           # 外部服务适配器
    ├── api/                # Messages API 客户端与 SSE 流式传输
    ├── commands/           # 斜杠命令注册与解析
    ├── compat-harness/     # 兼容性工具
    ├── embedding/          # 语义嵌入向量模型
    ├── intent/             # 意图推断与匹配
    ├── knowledge/          # 知识库与 RAG
    ├── llm/                # LLM 客户端与类型
    ├── memory/             # 记忆系统 (Hebbian + Tier)
    ├── model-router/       # 智能模型路由器
    ├── patent-domain/      # 专利领域服务
    ├── reasoning/          # 推理引擎
    ├── router/             # 工作流路由
    ├── runtime/            # 会话、配置、权限、MCP、提示词管理
    ├── server/             # HTTP 服务
    ├── tools/              # 内置工具实现
    ├── workflow/           # 工作流引擎
    └── yunxi-cli/          # 主 CLI + 全屏 TUI（yunxi）
```

### 各模块职责

- **adapters** — 外部服务适配器（CNIPA 检索等）
- **api** — HTTP 客户端、SSE 流解析、请求/响应类型、认证（API Key + OAuth）
- **commands** — 斜杠命令定义、解析、补全与帮助文本生成
- **compat-harness** — 上游源码工具/提示词清单提取
- **embedding** — 语义嵌入向量生成与检索（BGE-M3 等）
- **intent** — 自然语言→命令意图匹配
- **knowledge** — 知识库构建与 RAG 检索
- **llm** — LLM API 客户端、流式响应、Token 估价
- **memory** — Hebbian 记忆系统 + 4 层 Tier 存储
- **model-router** — 根据任务复杂度自动选择模型（pro/flash）
- **patent-domain** — 专利领域服务（分类、检索、对比）
- **reasoning** — 多步推理引擎
- **router** — 工作流路由与工具推荐
- **runtime** — 对话运行时、配置加载层级、会话持久化、权限策略、MCP 客户端、系统提示词
- **server** — Axum HTTP 服务端点
- **tools** — 内置工具集（Bash、ReadFile、WriteFile、EditFile、Grep、Glob、WebSearch、WebFetch、Agent、TodoWrite 等）
- **workflow** — 工作流编排引擎
- **yunxi-cli** — 全屏 TUI、REPL 交互、斜杠命令路由、Markdown 终端渲染

---

## 快速开始

### 构建

```bash
cd rust/
cargo build --release
```

### 运行

```bash
# 交互式 REPL
./target/release/yunxi

# 单次提示
./target/release/yunxi prompt "分析这份专利的权利要求"

# 指定模型（默认 deepseek-v4-pro）
./target/release/yunxi --model deepseek prompt "检索相关技术领域的专利"
```

### 配置

设置 API 凭据（按所用模型选择）：

```bash
# DeepSeek（默认推荐）
export DEEPSEEK_API_KEY="sk-..."

# 通用云熙凭据（Messages API / OAuth）
export YUNXI_API_KEY="..."
export YUNXI_API_BASE_URL="https://your-gateway.example"
```

配置目录默认为 `~/.yunxi/`，项目级见 `.yunxi.json` 与 `.yunxi/settings.local.json`。

或通过 OAuth 认证：

```bash
yunxi login
```

---

## 功能特性

| 功能 | 状态 |
|------|------|
| Messages API + 流式传输 | ✅ |
| OAuth 登录/登出 | ✅ |
| 交互式 REPL（rustyline） | ✅ |
| 工具系统（bash、read、write、edit、grep、glob） | ✅ |
| Web 工具（search、fetch） | ✅ |
| 子智能体编排 | ✅ |
| 待办事项追踪 | ✅ |
| Notebook 编辑 | ✅ |
| YUNXI.md / 项目记忆 | ✅ |
| 配置文件层级（.yunxi.json） | ✅ |
| 权限系统 | ✅ |
| MCP 服务器接入 | ⚠️ 仅 Stdio 可用 |
| 会话持久化与恢复 | ✅ |
| 扩展思考（thinking blocks） | ✅ |
| 费用追踪与用量显示 | ✅ |
| Git 集成 | ✅ |
| Markdown 终端渲染（ANSI） | ✅ |
| 模型别名（deepseek / messages-*） | ✅ |
| 斜杠命令（/status、/compact、/clear 等） | ✅ |
| 钩子（PreToolUse/PostToolUse） | 🔧 仅配置 |
| 技能注册表（26 个专利技能） | ✅ |
| 插件系统 | 📋 计划中 |

## 模型别名

| 别名 | 对应模型 |
|------|---------|
| `auto` | 智能路由，根据任务复杂度自动选择 |
| `deepseek` / `ds` | `deepseek-v4-pro` |
| `deepseek-flash` / `dsf` | `deepseek-v4-flash` |
| `messages-opus` | `messages-opus` |
| `messages-sonnet` | `messages-sonnet` |
| `messages-haiku` | `messages-haiku` |

### 智能模型路由器

YunXi 支持智能模型路由功能，通过 `--model auto` 或配置 `"model": "auto"` 启用。系统根据任务复杂度自动选择最合适的模型：

- **规划、分析、设计** 等复杂任务 → `deepseek-v4-pro`
- **日常聊天、执行、修改** 等简单任务 → `deepseek-v4-flash`

**评分维度（总分100，阈值65）：**
- 任务类型（40%）：规划/分析/生成/执行/聊天
- 输入复杂度（20%）：长度、代码、结构化数据
- 上下文复杂度（20%）：历史对话、涉及文件
- 工具调用（20%）：预估次数、复杂工具

配置示例见 `.yunxi/settings.json.example`。

## CLI 参数

```bash
yunxi [选项] [命令]

选项：
  --model MODEL                    设置模型（别名或全名）
  --dangerously-skip-permissions   跳过所有权限检查
  --permission-mode MODE           设置权限模式（read-only、workspace-write、danger-full-access）
  --allowedTools TOOLS             限制启用的工具
  --output-format FORMAT           输出格式（text 或 json）
  --version, -V                    显示版本信息

命令：
  prompt <文本>      单次提示（非交互式）
  login              通过 OAuth 认证
  logout             清除已保存的凭据
  init               初始化项目配置
  doctor             检查环境健康状态
  self-update        更新到最新版本
```

## 斜杠命令（TUI / REPL）

| 命令 | 说明 |
|------|------|
| `/help` | 显示可用斜杠命令和快捷键 |
| `/status` | 显示会话状态（模型、Token、费用、工作流等） |
| `/cost` | 显示费用明细 |
| `/compact` | 压缩对话历史 |
| `/clear [--confirm]` | 清空对话 |
| `/model [名称]` | 显示或切换模型 |
| `/permissions [模式]` | 显示或切换权限模式（read-only/workspace-write/danger-full-access） |
| `/config [env\|hooks\|model]` | 查看配置项 |
| `/memory` | 查看已加载的指导记忆文件 |
| `/init` | 为当前目录生成 YUNXI.md |
| `/diff` | 显示工作区 git diff（彩色） |
| `/export [路径]` | 导出对话到文件 |
| `/search <关键词>` | 检索对话历史 |
| `/session [list\|switch <ID>]` | 列出或切换本地会话 |
| `/version` | 显示 CLI 版本和构建信息 |
| `/undo` | 撤销上一次交互 |
| `/resume <会话路径>` | 加载已保存的会话 |
| `/bughunter [范围]` | 检查代码库中的潜在缺陷 |
| `/commit` | 生成提交信息并创建 git commit |
| `/pr [上下文]` | 基于对话起草 Pull Request |
| `/issue [上下文]` | 基于对话起草 GitHub Issue |
| `/ultraplan [任务]` | 运行深度规划提示词（多步推理） |
| `/teleport <符号或路径>` | 跳转到文件或符号 |
| `/debug-tool-call` | 回放上次工具调用并显示调试详情 |
| `/semantic` | 查看语义嵌入与索引状态 |
| `/flow [list\|resume\|clear]` | 管理工作流挂起 |

### 专利专屏斜杠命令（`yunxi --patent`）

| 命令 | 说明 |
|------|------|
| `/init` | 扫描文件夹并更新 YUNXI.md |
| `/extract [路径]` | 批量抽取办公文件并灌入各视图 |
| `/preview <路径>` | 单文件抽取预览（不灌入） |
| `/ocr <路径>` | 单文件 OCR/抽取并灌入主视图 |
| `/panel` | 分页查看当前主视图全文 |
| `/materials` | 材料清单与工具建议 |
| `/view <1-6\|名称>` | 切换主视图（权利要求/对比/审查意见/检索/草稿/对话） |
| `/case [set key=val]` | 查看或设置案件信息 |
| `/import <路径>` | 导入文本或办公文件 |
| `/export [md\|docx]` | 国知局版式导出 |
| `/reload` | 从 YUNXI.md 重载案件信息 |

---

## Python 工作区

项目同时包含 Python 工作区用于辅助开发：

```text
├── src/                                # Python 工作区
│   ├── __init__.py
│   ├── commands.py                     # 命令端口元数据
│   ├── main.py                         # CLI 入口
│   ├── models.py                       # 数据模型
│   ├── port_manifest.py                # 工作区结构清单
│   ├── query_engine.py                 # 移植摘要渲染
│   ├── task.py                         # 任务模型
│   └── tools.py                        # 工具端口元数据
├── tests/                              # Python 验证
└── assets/                             # 品牌素材
```

### Python 快速命令

```bash
# 渲染移植摘要
python3 -m src.main summary

# 打印工作区清单
python3 -m src.main manifest

# 列出子系统
python3 -m src.main subsystems --limit 16

# 运行验证
python3 -m unittest discover -s tests -v
```

---

## 统计信息

- **约 2 万行** Rust 代码
- **18 个** workspace crate
- **二进制名称：** `yunxi`
- **默认模型：** `deepseek-v4-pro`
- **默认权限：** `danger-full-access`

## 许可证

MIT

---

## 声明

