# YunXi 本机使用指南

面向**单机本地开发**（无远程仓库也可使用）。按顺序完成即可跑通 L0–L4 能力。

## 1. 前置依赖

| 组件 | 用途 | 安装 |
|------|------|------|
| Rust stable | 编译 `yunxi` | [rustup.rs](https://rustup.rs) |
| Python 3.10+ | 辅助脚本与 markitdown | 系统自带或 Homebrew |
| DEEPSEEK_API_KEY | LLM 对话（默认模型） | 环境变量或 `~/.yunxi/` |
| oMLX :8009 | BGE 嵌入 / 混合语义检索 | 本机启动 oMLX |
| 知识库资产 | 图谱 / 法规 / 语义索引 | 见下文「资产」 |

## 2. 构建 CLI

```bash
cd rust
export CARGO_TARGET_DIR="$(pwd)/target"   # 建议：二进制落在仓库内
cargo build --release
export PATH="$(pwd)/target/release:$PATH"
yunxi --version
```

## 3. 环境检查

```bash
yunxi doctor
```

检查项包括：LLM 密钥、知识库 SQLite、`127.0.0.1:8009`、markitdown、poppler。

## 4. 配置

**推荐**将密钥与语义服务配置放在用户目录，勿写入项目仓库：

```bash
mkdir -p ~/.yunxi
cp .yunxi/settings.semantic.example.json ~/.yunxi/settings.local.json
# 编辑 apiKey，或 export OMLX_API_KEY=...
export DEEPSEEK_API_KEY="sk-..."
```

项目级 [`.yunxi/settings.json`](.yunxi/settings.json) 可保留模型与权限；本地覆盖用 `~/.yunxi/settings.local.json` 或项目 `.yunxi/settings.local.json`（勿提交密钥）。

## 5. 知识库资产（本机）

大文件**默认不在 Git** 中（见 [`.gitignore`](.gitignore)、[docs/GIT_REPOSITORY.md](docs/GIT_REPOSITORY.md)）。你需在本机具备：

| 文件 | 路径 |
|------|------|
| 专利知识图谱 | `assets/knowledge_graph/patent_kg.db` |
| 语义向量索引 | `assets/knowledge-base/.yunpat-semantic-index.sqlite` |
| 法律法规 | `assets/knowledge/data/laws.db` |
| Markdown 知识库 | `assets/knowledge-base/**` |

### 从 Obsidian / YunPat 同步

```bash
# 预览
./scripts/sync-knowledge-base.sh --dry-run

# 实际同步（需本机 Obsidian 库与可选 yunpat-agent 路径）
./scripts/sync-knowledge-base.sh
```

环境变量见脚本头部：`YUNXI_OBSIDIAN_VAULT`、`YUNXI_YUNPAT_KB`、`YUNXI_KB_DEST`、`YUNXI_GRAPH_DEST`。

**注意**：代码唯一引用 `assets/knowledge_graph/patent_kg.db`。若 `knowledge-base/patent_kg.db` 较新，请执行：

```bash
cp assets/knowledge-base/patent_kg.db assets/knowledge_graph/patent_kg.db
```

`yunxi doctor` 会检查两份文件是否不同步。也可设置环境变量 `PATENT_KG_DB` 指向自定义路径。

## 6. 分层验收

### L0 — 开发与回归（无需网络）

```bash
cd rust && cargo fmt -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cd .. && ruff check src/ tests/ && python3 -m pytest tests/ -v
```

### L1 — CLI 壳

```bash
yunxi init
yunxi          # 进入 TUI
```

### L2 — LLM 对话

```bash
yunxi prompt "用一句话说明专利法的目的"
# 或显式：yunxi --model deepseek prompt "..."
```

### L3 — 知识库与语义

1. 启动 oMLX（默认 `http://127.0.0.1:8009`）
2. TUI 中执行 `/semantic` 查看索引状态
3. 在对话中触发知识检索类工具

### L4 — 专利办公链

```bash
pip install 'markitdown[pdf,docx,pptx]'
brew install poppler    # 扫描 PDF
# 可选：brew install tesseract tesseract-lang
```

办公抽取脚本：[`scripts/patent/README.md`](scripts/patent/README.md)

专利案件请用桌面客户端 `yunxi-desktop`（见下文 §7）；`yunxi --patent` 已移除并会提示改用桌面端。办公抽取仍可用 `scripts/patent/markitdown_convert.py` 做 L4 依赖验证。

## 7. macOS 桌面客户端（Tauri）

设计规范与进度见 [`rust/crates/yunxi-cli/DESKTOP-DEVELOPMENT-LOG.md`](rust/crates/yunxi-cli/DESKTOP-DEVELOPMENT-LOG.md)。

```bash
cd rust
./crates/yunxi-cli/bundle-desktop.sh          # 打包 .app 并打开（推荐）
# 或
cd crates/yunxi-cli/frontend
npm run bundle:desktop && npm run open:desktop
```

- **TUI 命令**：`yunxi`（`--bin yunxi`），不是 `yunxi-desktop`
- **桌面 GUI**：用 `.app` 启动才有正确 Dock 图标
- **配置**：`~/.yunxi/settings.local.json` 与项目 `.yunxi/settings.local.json` 均已支持

## 8. 已知限制

| 能力 | 状态 |
|------|------|
| `yunxi doctor` | 已实现 |
| `yunxi self-update` | 已实现（在仓库内 `cargo build --release` 并覆盖当前二进制） |
| `yunxi server`（HTTP API） | 已实现，默认 `127.0.0.1:8765`；`yunxi server --host HOST --port PORT` |
| Docker / 远程部署 | 未提供 |

## 9. 故障排除

- **Anthropic API key not found**：未设置 DeepSeek 密钥；使用 `export DEEPSEEK_API_KEY=...` 或 `yunxi --model deepseek`。
- **语义检索无结果**：确认 oMLX 运行且 `semantic.enabled` 为 true。
- **编译产物找不到**：设置 `CARGO_TARGET_DIR=rust/target` 后重新 `cargo build --release`。
- **桌面打包路径错误**：当前目录已在 `rust/` 时用 `crates/yunxi-cli/frontend`，勿写 `rust/crates/...`。
- **Dock 显示 exec 图标**：用 `bundle-desktop.sh` 或 `.app` 启动，不要直接运行 `target/release/yunxi-desktop`。
- **TUI 报未配置 API Key**：确认 `~/.yunxi/settings.local.json` 含 `env.DEEPSEEK_API_KEY`；运行 `yunxi doctor` 自检。
