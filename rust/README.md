<p align="center">
  <img src="../assets/logo.svg" alt="云熙智能体 Logo" width="160" />
</p>

<h1 align="center">云熙智能体 — Rust 实现</h1>

<p align="center">
  <strong>YunXi Agent — 专业专利智能体 · Rust 实现</strong>
</p>

<p align="center">
  基于 Rust 构建的高性能专利智能处理引擎。为速度、安全和原生工具执行而生。
</p>

---

## 快速开始

```bash
# 构建
cd rust/
cargo build --release

# 运行交互式 REPL
./target/release/yunxi

# 单次提示
./target/release/yunxi prompt "分析这份专利的权利要求"

# 指定模型
./target/release/yunxi --model sonnet prompt "检索相关技术领域的专利"
```

## 配置

设置 API 凭据：

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# 或使用代理
export ANTHROPIC_BASE_URL="https://your-proxy.com"
```

或通过 OAuth 认证：

```bash
yunxi login
```

## 功能特性

| 功能 | 状态 |
|------|------|
| Anthropic API + 流式传输 | ✅ |
| OAuth 登录/登出 | ✅ |
| 交互式 REPL（rustyline） | ✅ |
| 工具系统（bash、read、write、edit、grep、glob） | ✅ |
| Web 工具（search、fetch） | ✅ |
| 子智能体编排 | ✅ |
| 待办事项追踪 | ✅ |
| Notebook 编辑 | ✅ |
| CLAUDE.md / 项目记忆 | ✅ |
| 配置文件层级（.claude.json） | ✅ |
| 权限系统 | ✅ |
| MCP 服务器生命周期 | ✅ |
| 会话持久化与恢复 | ✅ |
| 扩展思考（thinking blocks） | ✅ |
| 费用追踪与用量显示 | ✅ |
| Git 集成 | ✅ |
| Markdown 终端渲染（ANSI） | ✅ |
| 模型别名（opus/sonnet/haiku） | ✅ |
| 斜杠命令（/status、/compact、/clear 等） | ✅ |
| 钩子（PreToolUse/PostToolUse） | 🔧 仅配置 |
| 插件系统 | 📋 计划中 |
| 技能注册表 | 📋 计划中 |

## 模型别名

| 别名 | 对应模型 |
|------|---------|
| `opus` | `claude-opus-4-6` |
| `sonnet` | `claude-sonnet-4-6` |
| `haiku` | `claude-haiku-4-5-20251213` |

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

## 斜杠命令（REPL）

| 命令 | 说明 |
|------|------|
| `/help` | 显示帮助 |
| `/status` | 显示会话状态（模型、Token、费用） |
| `/cost` | 显示费用明细 |
| `/compact` | 压缩对话历史 |
| `/clear` | 清空对话 |
| `/model [名称]` | 显示或切换模型 |
| `/permissions` | 显示或切换权限模式 |
| `/config [分区]` | 显示配置（env、hooks、model） |
| `/memory` | 显示 CLAUDE.md 内容 |
| `/diff` | 显示 git diff |
| `/export [路径]` | 导出对话 |
| `/session [id]` | 恢复之前的会话 |
| `/version` | 显示版本 |

## 工作区结构

```text
rust/
├── Cargo.toml              # Workspace 根配置
├── Cargo.lock
└── crates/
    ├── api/                # Anthropic API 客户端与 SSE 流式传输
    ├── commands/           # 斜杠命令注册与解析
    ├── compat-harness/     # 兼容性工具
    ├── runtime/            # 会话、配置、权限、MCP、提示词管理
    ├── yunxi-cli/          # 主 CLI 二进制（yunxi）
    └── tools/              # 内置工具实现
```

### 各 crate 职责

- **api** — HTTP 客户端、SSE 流解析、请求/响应类型、认证（API Key + OAuth Bearer）
- **commands** — 斜杠命令定义与帮助文本生成
- **compat-harness** — 提取上游 TS 源码的工具/提示词清单
- **runtime** — 对话运行时、配置加载层级、会话持久化、权限策略、MCP 客户端、系统提示词组装、用量追踪
- **yunxi-cli** — REPL 交互、单次提示、流式显示、工具调用渲染、CLI 参数解析
- **tools** — 工具规格与执行：Bash、ReadFile、WriteFile、EditFile、GlobSearch、GrepSearch、WebSearch、WebFetch、Agent、TodoWrite、NotebookEdit、Skill、ToolSearch、REPL 运行时

## 统计信息

- **约 2 万行** Rust 代码
- **6 个** workspace crate
- **二进制名称：** `yunxi`
- **默认模型：** `claude-opus-4-6`
- **默认权限：** `danger-full-access`

## 许可证

见仓库根目录。
