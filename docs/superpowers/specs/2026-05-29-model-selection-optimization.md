# 模型选择优化设计

日期: 2026-05-29
状态: 待实施

## 问题

1. `/model` 斜杠命令补全时显示了过多不必要的选项（别名 `ds`/`dsf`/`deepseek`/`deepseek-flash` 和非 DeepSeek 的 `messages-*`）
2. `auto` 模式在 TUI 中不工作 — `tui/runner.rs` 的 `start_turn()` 完全没有 auto 路由逻辑
3. 缺少 auto 路由的可视化反馈

## 方案

### 1. 精简模型候选列表

**文件**: `rust/crates/yunxi-cli/src/slash_complete_shared.rs`

将 `MODEL_CANDIDATES` 精简为 3 项：

```rust
const MODEL_CANDIDATES: &[&str] = &[
    "auto",
    "deepseek-v4-flash",
    "deepseek-v4-pro",
];
```

别名 `ds`/`dsf`/`deepseek`/`deepseek-flash` 仍通过 `resolve_model_alias()` 支持，只是不在补全菜单中显示。

### 2. TUI 中实现 auto 路由

**文件**: `rust/crates/yunxi-cli/src/tui/runner.rs`

在 `start_turn()` 中注入 auto 路由逻辑（对齐 `live_cli.rs:127-157`）：

```
start_turn() 流程:
1. 获取 app.model()（可能是 "auto" 或具体模型名）
2. 如果是 "auto"，调用 select_model_for_request() 解析
3. 如果解析结果与 active_model 不同，重建 runtime
4. 用解析后的模型发起请求
```

**文件**: `rust/crates/yunxi-cli/src/tui/runner.rs`

`TuiState` 新增 `active_model: Option<String>` 字段，跟踪 auto 解析后的实际模型。

### 3. 状态栏增强

**文件**: `rust/crates/yunxi-cli/src/tui/status_bar.rs`

当配置模型为 `auto` 时，状态栏模型区域显示 `auto→pro` 或 `auto→flash`。

### 4. `/status` 显示路由信息

当模型为 auto 时，`/status` 输出中额外显示路由配置和最近的路由决策。

### 5. `/model` 裸命令改进

当 `/model` 不带参数时，显示简洁的可选模型列表和当前使用的模型。

## 涉及文件

| 文件 | 改动 |
|---|---|
| `slash_complete_shared.rs` | 精简 `MODEL_CANDIDATES`，加入 `auto` |
| `tui/runner.rs` | `start_turn()` 注入 auto 路由 |
| `tui/slash.rs` | `/model` 裸命令改进 |
| `tui/status_bar.rs` | auto 模式下状态栏增强 |
| `tui/app.rs` | 新增 `active_model` 状态 |
| `format_report.rs` | `/status` auto 路由信息 |

## 验证

1. `/model ` 补全只显示 `auto`/`deepseek-v4-flash`/`deepseek-v4-pro`
2. `/model auto` 后，发送简单消息（"你好"）应使用 flash
3. `/model auto` 后，发送复杂消息（"帮我规划架构并评估风险"）应使用 pro
4. 状态栏在 auto 模式下显示 `auto→flash` 或 `auto→pro`
5. `cargo test --workspace` 通过
6. `cargo clippy --workspace --all-targets -- -D warnings` 通过
