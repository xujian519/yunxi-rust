# TUI 评审问题修复计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 全面修复 2026-05-29 TUI 评审发现的 7 个问题，确保项目可编译、文档准确、代码质量达标。

**Architecture:** 分 7 个独立任务，涉及 model-router crate 源码创建、README 文档更新、Cargo.toml 配置修正、帮助层文案修正、工具 crate clippy 修复。

**Tech Stack:** Rust workspace (18 crates), Cargo.toml lint 配置

---

### Task 1: 补齐 model-router crate 源码

**Files:**
- Create: `rust/crates/model-router/src/lib.rs`

**背景:** 最新 commit `c951639` 创建了 model-router crate 但只有 `Cargo.toml`，无 `src/lib.rs`，导致整个 workspace 无法编译。当前存在一个临时 stub，需要替换为正式内容。

**Step 1: 查看 model-router 设计文档了解其用途**

```bash
grep -r "model-router\|model_router\|ModelRouter" docs/ --include="*.md" | head -20
```

**Step 2: 创建正式版 lib.rs**

在 `rust/crates/model-router/src/lib.rs` 中创建模块框架：

```rust
//! 智能模型路由器 — 根据任务复杂度自动选择模型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub default_model: String,
    pub fallback_model: String,
    pub complexity_rules: Vec<ComplexityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRule {
    pub pattern: String,
    pub min_tokens: Option<u32>,
    pub model: String,
}

#[derive(Debug, Clone, Default)]
pub struct RouteDecision {
    pub model: String,
    pub reason: String,
    pub complexity_score: f32,
}

impl RouteDecision {
    pub fn new(model: String, reason: String, complexity_score: f32) -> Self {
        Self {
            model,
            reason,
            complexity_score,
        }
    }
}

pub fn route_task(_input: &str, _config: &RouterConfig) -> RouteDecision {
    todo!("路由算法待实现 — 参见 docs/plans/ 中的 model-router 设计文档")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_decision_default() {
        let decision = RouteDecision::default();
        assert!(decision.model.is_empty());
    }

    #[test]
    fn route_decision_new() {
        let d = RouteDecision::new("deepseek-v4-flash".into(), "简单任务".into(), 0.2);
        assert_eq!(d.model, "deepseek-v4-flash");
        assert_eq!(d.reason, "简单任务");
    }
}
```

**Step 3: 编译验证**

```bash
cd rust && cargo check -p model-router 2>&1
```
预期: 零错误通过

**Step 4: 提交**

```bash
git add rust/crates/model-router/src/lib.rs
git commit -m "feat(model-router): 创建 src/lib.rs，解除 workspace 编译阻塞"
```

---

### Task 2: 更新 README.md 斜杠命令表

**Files:**
- Modify: `README.md` (第 162-180 行斜杠命令表区域)

**Step 1: 对照实际代码确认完整命令列表**

实际 24 个命令（来源: `rust/crates/commands/src/lib.rs`），README 只列出 15 个。

**Step 2: 替换斜杠命令表**

将第 162 行开始的 `## 斜杠命令（REPL）` 区域替换为：

```markdown
## 斜杠命令（TUI / REPL）

| 命令 | 说明 |
|------|------|
| `/help` | 显示可用斜杠命令和快捷键 |
| `/status` | 显示会话状态（模型、Token、费用、工作流等） |
| `/cost` | 显示费用明细 |
| `/compact` | 压缩对话历史 |
| `/clear [--confirm]` | 清空对话 |
| `/model [名称]` | 显示或切换模型 |
| `/permissions [模式]` | 显示或切换权限模式 |
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

### 专利专屏斜杠命令（`yunxi --patent`）

| 命令 | 说明 |
|------|------|
| `/init` | 扫描文件夹并更新 YUNXI.md |
| `/extract [路径]` | 批量抽取办公文件并灌入各视图 |
| `/preview <路径>` | 单文件抽取预览（不灌入） |
| `/ocr <路径>` | 单文件 OCR/抽取并灌入主视图 |
| `/panel` | 分页查看当前主视图全文 |
| `/materials` | 材料清单与工具建议 |
| `/view <1-6\|名称>` | 切换主视图 |
| `/case [set key=val]` | 查看或设置案件信息 |
| `/import <路径>` | 导入文本或办公文件 |
| `/export [md\|docx]` | 国知局版式导出 |
| `/reload` | 从 YUNXI.md 重载案件信息 |
```

**Step 3: 修正架构说明中的 crate 数量**

第 221 行 `**6 个** workspace crate` 改为 `**18 个** workspace crate`

**Step 4: 提交**

```bash
git add README.md
git commit -m "docs(readme): 更新斜杠命令表和架构信息"
```

---

### Task 3: 修复 Cargo.toml 中过时的 lint 名称

**Files:**
- Modify: `rust/Cargo.toml` (第 38, 50, 51, 57, 60 行)

**Step 1: 定位并替换过时的 lint 名**

```toml
# 第 38 行: useless_let_and_seq → useless_let_if_seq
```
将 `useless_let_and_seq = "allow"` 改为 `useless_let_if_seq = "allow"`

```toml
# 第 50 行: replace_with_itself → 删除
```
删除 `replace_with_itself = "allow"` 这一行（此 lint 不存在）

```toml
# 第 51 行: format_in_format_out → format_in_format_args
```
将 `format_in_format_out = "allow"` 改为 `format_in_format_args = "allow"`

```toml
# 第 57 行: identity_conversion → useless_conversion
```
将 `identity_conversion = "allow"` 改为 `useless_conversion = "allow"`

```toml
# 第 60 行: unused_variables → 删除
```
删除 `unused_variables = "allow"` 这一行（这是 rustc lint，不是 clippy lint；Cargo.toml 中已通过 `[workspace.lints.rust]` 的 `dead_code = "allow"` 放宽）

**Step 2: 验证 clippy 警告消除**

```bash
cd rust && cargo clippy --workspace 2>&1 | grep "unknown lint"
```
预期: 无输出（所有未知 lint 警告消除）

**Step 3: 提交**

```bash
git add rust/Cargo.toml
git commit -m "fix(lint): 修复 Cargo.toml 中 5 个过时的 clippy lint 名称"
```

---

### Task 4: 修复帮助层中 Tab 补全模式提示

**Files:**
- Modify: `rust/crates/yunxi-cli/src/tui/components/help_overlay.rs:54`

**Step 1: 修改提示文字**

当前代码 (第 53-54 行):
```rust
lines.push("\x1b[2m  输入 / 后可 Tab 补全（--repl 模式）\x1b[0m".to_string());
```

改为:
```rust
lines.push("\x1b[2m  输入 / 后可 Tab 补全（TUI 和 REPL 均支持）\x1b[0m".to_string());
```

**Step 2: 运行测试**

```bash
cd rust && cargo test -p yunxi-cli help_overlay 2>&1
```
预期: 4 个测试全部通过（含 `help_overlay_renders`）

**Step 3: 提交**

```bash
git add rust/crates/yunxi-cli/src/tui/components/help_overlay.rs
git commit -m "fix(tui): 修正帮助层中 Tab 补全的模式描述"
```

---

### Task 5: TUI 增加 /exit 和 /quit 命令支持

**Files:**
- Modify: `rust/crates/yunxi-cli/src/tui/slash.rs` (在命令解析之前添加 exit/quit 拦截)

**Step 1: 在 slash.rs 中添加 exit/quit 处理**

在 `handle_slash_command` 函数中，`Semantic` 处理之前（约第 52 行前）插入：

```rust
if matches!(input.trim(), "/exit" | "/quit") {
    app.push_system_message("使用 q 键或 Ctrl+C 退出 TUI 模式。");
    return Ok(Some(SlashDispatch::Handled));
}
```

**Step 2: 运行测试**

```bash
cd rust && cargo test -p yunxi-cli 2>&1
```
预期: 196 个测试全部通过

**Step 3: 提交**

```bash
git add rust/crates/yunxi-cli/src/tui/slash.rs
git commit -m "feat(tui): TUI 模式支持 /exit 和 /quit 提示退出方式"
```

---

### Task 6: 修复 tools crate 的 6 个 clippy 错误

**Files:**
- Modify: `rust/crates/tools/src/code_eval/style.rs:45`
- Modify: `rust/crates/tools/src/eval_framework/registry.rs:26`
- Modify: `rust/crates/tools/src/llm_eval/g_eval.rs:36`
- Modify: `rust/crates/tools/src/quality_gate/mod.rs:211-212`
- Modify: `rust/crates/tools/src/reflection/llm_reflection.rs:117`

**Step 1: 读取每个文件获取完整上下文再修复**

**Style.rs (look-around regex):**
将 `Regex::new(r"\b(?!0\b)(?!1\b)\d{2,}\b")` 替换为不使用 look-ahead 的等价写法：
```rust
Regex::new(r"\b([2-9]|\d{3,})\b")
```
(因为 `(?!0\b)(?!1\b)\d{2,}` 匹配不以 0 或 1 开头的 2+ 位数字，等价于 2-9 开头的 2 位及以上数字)

**Registry.rs (&Box<dyn Evaluator>):**
将返回类型 `Option<&Box<dyn Evaluator>>` 改为 `Option<&dyn Evaluator>`，并在实现中使用 `map(|b| b.as_ref())`。

**G_eval.rs (useless format!):**
将 `let mut prompt = format!("固定字符串");` 改为 `let mut prompt = "固定字符串".to_string();`

**Quality_gate/mod.rs (op_ref):**
将 `(metric_value - &threshold.value)` 改为 `(metric_value - threshold.value)`（两处）

**Llm_reflection.rs (useless format!):**
将 `let llm_output = format!("...");` 改为 `let llm_output = "...".to_string();`（clippy 已给出完整的帮助提示）

**Step 2: 验证所有 clippy 错误消除**

```bash
cd rust && cargo clippy -p tools -- -D warnings 2>&1
```
预期: 零错误通过

**Step 3: 提交**

```bash
git add rust/crates/tools/src/
git commit -m "fix(tools): 修复 tools crate 中 6 个 clippy 错误"
```

---

### Task 7: 最终验证

**Step 1: 全 workspace 编译检查**

```bash
cd rust && cargo check --workspace 2>&1
```
预期: 零错误

**Step 2: 全 workspace 测试**

```bash
cd rust && cargo test --workspace 2>&1
```
预期: 全部通过

**Step 3: 全 workspace clippy**

```bash
cd rust && cargo clippy --workspace -- -D warnings 2>&1
```
预期: 零错误，仅允许的 warn 级别提示

**Step 4: 提交（如有未提交的变更）**

```bash
git status
git diff --stat
```
