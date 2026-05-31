# TUI 四项改进实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 TUI 的四个潜在改进点：专利屏 Pager 覆盖层渲染、Home/End 键支持、极端尺寸布局防护、消除参数解析冗余。

**Architecture:** 全部改动集中在 `rust/crates/yunxi-cli/src/tui/` 和 `cli_action.rs`。每个 Task 独立可测试，按风险从低到高排序。

**Tech Stack:** Rust, ratatui 0.29, crossterm 0.28

---

### Task 1: 输入框 Home/End 键支持

**Files:**
- Modify: `rust/crates/yunxi-cli/src/tui/app.rs` — KeyEvent 枚举 + handle_key 分支 + convert_key 映射
- Modify: `rust/crates/yunxi-cli/src/tui/components/input_bar.rs` — move_home / move_end 方法
- Test: `rust/crates/yunxi-cli/src/tui/components/input_bar.rs` (inline tests)

- [ ] **Step 1: 在 InputBar 添加 move_home / move_end 方法**

在 `input_bar.rs` 的 `impl InputBar` 中，紧跟 `move_right` 之后添加两个方法：

```rust
pub(crate) fn move_home(&mut self) {
    self.cursor = 0;
}

pub(crate) fn move_end(&mut self) {
    self.cursor = self.content.len();
}
```

- [ ] **Step 2: 在 KeyEvent 枚举添加 Home / End 变体**

在 `app.rs` 的 `KeyEvent` 枚举中，`Tab` 之后添加：

```rust
Home,
End,
```

- [ ] **Step 3: 在 handle_key 中处理 Home / End**

在 `app.rs` 的 `handle_key` 方法的 `match key` 块中，在 `KeyEvent::Left` 分支之前添加：

```rust
KeyEvent::Home => {
    self.input.move_home();
    None
}
KeyEvent::End => {
    self.input.move_end();
    None
}
```

- [ ] **Step 4: 在 runner.rs 的 convert_key 中映射 crossterm 按键**

在 `runner.rs` 的 `convert_key` 函数中，`KeyCode::Esc` 之前添加：

```rust
KeyCode::Home => KeyEvent::Home,
KeyCode::End => KeyEvent::End,
```

- [ ] **Step 5: 添加单元测试**

在 `input_bar.rs` 的 `mod tests` 中添加：

```rust
#[test]
fn input_bar_home_and_end() {
    let mut bar = InputBar::new();
    bar.insert('a');
    bar.insert('b');
    bar.insert('c');
    assert_eq!(bar.cursor, 3);
    bar.move_home();
    assert_eq!(bar.cursor, 0);
    bar.move_end();
    assert_eq!(bar.cursor, 3);
}
```

- [ ] **Step 6: 编译验证**

Run: `cd rust && cargo check --bin yunxi -p yunxi-cli`
Expected: 编译通过，无错误

- [ ] **Step 7: 运行测试**

Run: `cd rust && cargo test -p yunxi-cli input_bar_home_and_end -- --nocapture`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add rust/crates/yunxi-cli/src/tui/components/input_bar.rs rust/crates/yunxi-cli/src/tui/app.rs rust/crates/yunxi-cli/src/tui/runner.rs
git commit -m "feat(tui): 添加 Home/End 键支持输入框行首行尾跳转"
```

---

### Task 2: 极端尺寸布局防护

**Files:**
- Modify: `rust/crates/yunxi-cli/src/tui/layout.rs` — 最小尺寸校验 + 防护逻辑
- Test: `rust/crates/yunxi-cli/src/tui/layout.rs` (inline tests)

- [ ] **Step 1: 在 layout.rs 添加最小终端尺寸常量**

在 `layout.rs` 的 `input_block_height` 函数之前添加：

```rust
const MIN_WIDTH: u16 = 20;
const MIN_HEIGHT: u16 = 8;
```

- [ ] **Step 2: 修改 compute_with_input_rows 添加防护**

将 `compute_with_input_rows` 函数体的开头改为先 clamp 尺寸：

```rust
pub(crate) fn compute_with_input_rows(
    terminal_width: u16,
    terminal_height: u16,
    input_content_rows: u16,
    with_tool_panel: bool,
) -> Self {
    let terminal_width = terminal_width.max(MIN_WIDTH);
    let terminal_height = terminal_height.max(MIN_HEIGHT);
    let title_h = 1u16;
    let input_h = input_block_height(input_content_rows);
    let status_h = 1u16;

    let remaining = terminal_height
        .saturating_sub(title_h)
        .saturating_sub(input_h)
        .saturating_sub(status_h);

    if with_tool_panel && terminal_width >= 40 {
        let tool_w = std::cmp::max(20, terminal_width * 35 / 100);
        let chat_w = terminal_width.saturating_sub(tool_w);
        Self {
            title_bar: Rect::new(0, 0, terminal_width, title_h),
            chat_view: Rect::new(0, title_h, chat_w, remaining),
            tool_panel: Rect::new(chat_w, title_h, tool_w, remaining),
            input_bar: Rect::new(0, title_h + remaining, terminal_width, input_h),
            status_bar: Rect::new(0, title_h + remaining + input_h, terminal_width, status_h),
        }
    } else {
        Self {
            title_bar: Rect::new(0, 0, terminal_width, title_h),
            chat_view: Rect::new(0, title_h, terminal_width, remaining),
            tool_panel: Rect::ZERO,
            input_bar: Rect::new(0, title_h + remaining, terminal_width, input_h),
            status_bar: Rect::new(0, title_h + remaining + input_h, terminal_width, status_h),
        }
    }
}
```

关键变更：
- 入口 clamp 到 MIN_WIDTH/MIN_HEIGHT
- 工具面板条件增加 `terminal_width >= 40`（宽度不够时自动全宽）
- 空间不足时 remaining 可能为 0，chat_view 高度为 0 → is_valid() 返回 false，后续渲染自然跳过

- [ ] **Step 3: 同步修改 compute 和 compute_no_panel**

这两个便捷方法调用 `compute_with_input_rows`，无需额外修改，自动继承防护。

- [ ] **Step 4: 添加单元测试**

在 `layout.rs` 的 `mod tests` 中添加：

```rust
#[test]
fn layout_clamps_to_minimum_size() {
    let layout = Layout::compute(5, 3);
    assert!(layout.chat_view.is_valid() || layout.chat_view.height == 0);
    assert!(layout.title_bar.is_valid());
}

#[test]
fn layout_disables_tool_panel_on_narrow_width() {
    let layout = Layout::compute_with_input_rows(30, 20, 2, true);
    assert!(!layout.tool_panel.is_valid());
}

#[test]
fn layout_survives_zero_size() {
    let layout = Layout::compute(0, 0);
    assert!(layout.title_bar.width >= MIN_WIDTH);
    assert!(layout.title_bar.height >= MIN_HEIGHT);
}
```

- [ ] **Step 5: 编译验证**

Run: `cd rust && cargo check --bin yunxi -p yunxi-cli`
Expected: 编译通过

- [ ] **Step 6: 运行测试**

Run: `cd rust && cargo test -p yunxi-cli layout -- --nocapture`
Expected: 全部 PASS

- [ ] **Step 7: Commit**

```bash
git add rust/crates/yunxi-cli/src/tui/layout.rs
git commit -m "fix(tui): 极端终端尺寸布局防护，窄屏自动隐藏工具面板"
```

---

### Task 3: 专利屏 Pager 覆盖层渲染

**Files:**
- Modify: `rust/crates/yunxi-cli/src/tui/app.rs` — render_patent 中替换 pager 占位逻辑
- Test: `rust/crates/yunxi-cli/src/tui/app.rs` (inline tests)

- [ ] **Step 1: 修改 render_patent 方法，用 Frame overlay 渲染 pager**

将 `app.rs` 中 `render_patent` 方法的 pager 部分从：

```rust
let mut frame = render_patent_screen(&layout, &ctx, width, height);
if let Some(pager) = &self.pager {
    let overlay = Rect::new(width / 8, height / 6, width * 3 / 4, height * 2 / 3);
    let lines: Vec<String> = pager
        .render(overlay.height as usize)
        .lines()
        .map(String::from)
        .collect();
    // pager overlay via second pass would need Frame API — append hint in title bar for v1
    let _ = overlay;
    let _ = lines;
    frame.push_str("\n\x1b[2m[分页器打开 — Esc 关闭]\x1b[0m");
}
frame
```

替换为：

```rust
let mut frame_buf = Frame::new(width, height);
let patent_ansi = render_patent_screen(&layout, &ctx, width, height);
for (i, line) in patent_ansi.lines().enumerate() {
    if i >= height as usize {
        break;
    }
    frame_buf.set_row(i as u16, line);
}
if let Some(pager) = &self.pager {
    let overlay = Rect::new(width / 8, height / 6, width * 3 / 4, height * 2 / 3);
    let body = pager.render(overlay.height as usize);
    let lines: Vec<String> = body.lines().map(String::from).collect();
    frame_buf.overlay_lines(overlay, &lines);
}
frame_buf.as_ansi()
```

关键变更：
- 用 `Frame` 帧缓冲替代直接字符串拼接
- `Frame::overlay_lines` 已有实现，可将 pager 内容叠加到指定区域
- pager 不再是纯文本 hint，而是真正的覆盖层渲染

- [ ] **Step 2: 验证 app_ratatui.rs 中的 pager overlay**

`app_ratatui.rs:139-167` 已有 `render_pager_overlay` 的完整实现（通过 ratatui 的 Clear + Paragraph）。这是 ratatui 后端的 pager 渲染路径，不需要修改。

- [ ] **Step 3: 更新单元测试**

在 `app.rs` 的 `mod tests` 中修改 `app_render_produces_output` 测试，添加 pager 渲染验证：

```rust
#[test]
fn app_render_patent_with_pager_overlay() {
    let mut app = TuiApp::new(
        "deepseek-v4-pro".to_string(),
        "0.1.0".to_string(),
        UiMode::Patent,
    );
    app.push_output("Test Pager", "line 1\nline 2\nline 3", 80, 24);
    let rendered = app.render(80, 24);
    assert!(rendered.contains("Test Pager") || rendered.contains("分页器"));
}
```

- [ ] **Step 4: 编译验证**

Run: `cd rust && cargo check --bin yunxi -p yunxi-cli`
Expected: 编译通过

- [ ] **Step 5: 运行测试**

Run: `cd rust && cargo test -p yunxi-cli app_render_patent_with_pager -- --nocapture`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add rust/crates/yunxi-cli/src/tui/app.rs
git commit -m "fix(tui): 专利屏 Pager 覆盖层改为 Frame overlay 真正渲染"
```

---

### Task 4: 消除参数解析冗余（统一到 cli_action.rs）

**Files:**
- Modify: `rust/crates/yunxi-cli/src/args.rs` — 精简为仅导出类型定义
- Modify: `rust/crates/yunxi-cli/src/lib.rs` — 移除 args.rs 的 mod 引用（如不再需要）
- Modify: `rust/crates/yunxi-cli/Cargo.toml` — 移除 clap 依赖（如无其他使用者）

- [ ] **Step 1: 调查 args.rs 的实际使用情况**

Run: `cd rust && rg "use crate::args" crates/yunxi-cli/src/`
Expected: 确认哪些模块引用了 args

Run: `cd rust && rg "args::" crates/yunxi-cli/src/`
Expected: 确认具体引用的类型

- [ ] **Step 2: 评估是否可以安全移除 clap**

Run: `cd rust && rg "clap" crates/yunxi-cli/Cargo.toml crates/yunxi-cli/src/`
Expected: 确认 clap 仅被 args.rs 使用

根据初步调查：
- `args.rs` 定义了 `Cli`, `Command`, `PermissionMode`, `OutputFormat`，使用 clap derive
- `cli_action.rs` 定义了 `CliAction`, `CliOutputFormat`, `PermissionMode`，使用手写解析
- 两套类型有重叠但接口不同
- `args.rs` 的 `Cli` struct 在 `lib.rs` 中**未被使用**（parse_args 在 cli_action.rs 中手写）
- clap 可能仍被其他代码间接引用，需确认

如果 args.rs 确实无任何引用者，执行以下步骤。否则只添加弃用注释。

- [ ] **Step 3: 移除 args.rs 模块**

在 `lib.rs` 中删除 `mod args;` 行（如果存在）。

如果 args.rs 中有被其他模块使用的类型（如 PermissionMode），将它们迁移到 `cli_action.rs` 的对应位置或新建 `types.rs`。

- [ ] **Step 4: 移除 clap 依赖（如无其他使用者）**

在 `Cargo.toml` 中删除 `clap` 相关行。

- [ ] **Step 5: 编译验证**

Run: `cd rust && cargo check --bin yunxi -p yunxi-cli`
Expected: 编译通过

- [ ] **Step 6: 运行全部测试**

Run: `cd rust && cargo test -p yunxi-cli`
Expected: 全部 PASS

- [ ] **Step 7: Commit**

```bash
git add rust/crates/yunxi-cli/
git commit -m "refactor(tui): 消除 args.rs 参数解析冗余，统一使用 cli_action.rs"
```

---

## 自审清单

**1. Spec 覆盖：**
- ✅ Home/End 键 → Task 1
- ✅ 极端尺寸防护 → Task 2
- ✅ Pager 覆盖层 → Task 3
- ✅ 参数解析冗余 → Task 4

**2. 占位符扫描：** 无 TBD/TODO/placeholder。

**3. 类型一致性：**
- `KeyEvent` 新增 `Home`/`End` 在 app.rs 枚举定义和 runner.rs convert_key 中一致
- `InputBar` 的 `move_home`/`move_end` 方法签名与现有 `move_left`/`move_right` 一致
- `Frame` 已在 app.rs 中 import（`use crate::tui::frame::Frame`），render_patent 可直接使用
- `Rect::new` 参数顺序 (x, y, width, height) 与现有用法一致

**4. 风险评估：**
- Task 1-2：低风险，纯新增功能
- Task 3：中风险，替换渲染路径，需手动测试专利屏 pager
- Task 4：中风险，依赖调查结果，可能只需添加弃用注释而非删除
