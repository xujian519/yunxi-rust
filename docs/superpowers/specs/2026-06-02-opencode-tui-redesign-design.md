# YunXi TUI Opencode 复刻设计文档

**日期：** 2026-06-02
**版本：** 1.0
**策略：** 架构级重构 (方案 C)

## 📋 执行摘要

本文档详细说明了如何将 YunXi 的 TUI 界面完全复刻为 opencode 风格，采用架构级重构策略。这将涉及完全重新设计 TUI 架构，实现所有 16 项缺失功能，并建立可扩展的现代化组件库。

### 核心目标

- **像素级视觉复刻**：完全匹配 opencode 的外观和交互
- **架构现代化**：建立组件化、事件驱动的现代化架构
- **功能完整性**：实现所有 opencode TUI 功能
- **可扩展性**：为未来扩展预留架构空间
- **性能优化**：确保流畅的用户体验

### 时间估算

- **总计：** 8-12 周
- **阶段 1：** 2-3 周 (核心框架)
- **阶段 2：** 2-3 周 (组件库)
- **阶段 3：** 3-4 周 (高级功能)
- **阶段 4：** 1-2 周 (优化和测试)

---

## 🎯 功能需求分析

### 当前实现状态

#### ✅ 已实现功能 (4 项)

1. **基础布局**：标题栏 + 聊天区 + 工具面板 + 输入框 + 状态栏
2. **颜色系统**：品牌色 + 终端自适应
3. **消息显示**：用户/AI 消息气泡 + 角色标签
4. **滚动支持**：基础键盘滚动

#### ⚠️ 需要增强功能 (7 项)

1. **工具调用交互**：添加折叠/展开、图标、状态展示
2. **会话管理界面**：增强切换、删除、重命名体验
3. **权限确认体验**：详细说明 + 记忆选择
4. **代码语法高亮**：语言识别 + 语法着色
5. **思考过程展示**：可折叠思考块
6. **进度指示器多样性**：多种进度显示方式
7. **错误处理详情**：详细错误信息 + 重试机制

#### ❌ 完全缺失功能 (9 项)

1. **命令面板**：Ctrl+P 命令搜索和执行
2. **侧边栏导航系统**：可折叠侧边栏 + 多标签
3. **完整快捷键系统**：全面快捷键支持 + 自定义
4. **多主题支持**：主题切换 + 主题定制
5. **插件系统 UI**：插件面板 + 交互界面
6. **文件差异可视化**：git diff 可视化展示
7. **多选复制功能**：文本选择 + 批量复制
8. **工作区管理界面**：工作区切换 + 文件树
9. **对话复制功能**：消息复制 + 导出

---

## 🏗️ 架构设计

### 核心架构原则

1. **组件化**：每个 UI 元素都是独立组件
2. **事件驱动**：统一的 Action/Event 系统
3. **状态管理**：集中式全局状态 + 组件本地状态
4. **响应式布局**：自适应终端尺寸
5. **主题系统**：完整主题框架 + 动态切换

### 架构分层

```
┌─────────────────────────────────────┐
│   Application Layer (应用层)        │
│   - 主循环、事件循环、生命周期管理   │
├─────────────────────────────────────┤
│   Router Layer (路由层)              │
│   - 页面路由、状态机、导航          │
├─────────────────────────────────────┤
│   Components Layer (组件层)          │
│   - 可复用 UI 组件库                │
├─────────────────────────────────────┤
│   Renderer Layer (渲染层)            │
│   - ratatui 集成、缓冲区管理        │
├─────────────────────────────────────┤
│   State Layer (状态层)               │
│   - 全局状态、本地状态、事件系统    │
└─────────────────────────────────────┘
```

### 新目录结构

```
rust/crates/yunxi-cli/src/tui/
├── core/                    # 核心框架
│   ├── app.rs              # 应用主循环
│   ├── event.rs            # 事件系统
│   ├── action.rs           # Action 定义
│   ├── renderer.rs         # 渲染引擎
│   ├── lifecycle.rs        # 生命周期管理
│   └── mod.rs
├── state/                   # 状态管理
│   ├── global.rs           # 全局状态
│   ├── session.rs          # 会话状态
│   ├── theme.rs            # 主题状态
│   └── mod.rs
├── router/                  # 路由系统
│   ├── mod.rs
│   ├── routes.rs           # 路由定义
│   ├── home.rs             # 首页路由
│   └── session.rs          # 会话路由
├── components/              # 组件库
│   ├── mod.rs
│   ├── layout/             # 布局组件
│   ├── input/              # 输入组件
│   ├── feedback/           # 反馈组件
│   ├── navigation/         # 导航组件
│   ├── data/               # 数据组件
│   └── dialog/             # 对话框组件
├── views/                   # 视图
│   ├── mod.rs
│   ├── home.rs
│   ├── session.rs
│   └── workspace.rs
├── theme/                   # 主题系统
│   ├── mod.rs
│   ├── colors.rs
│   ├── palette.rs
│   └── themes.rs
├── keymap/                  # 快捷键系统
│   ├── mod.rs
│   ├── bindings.rs
│   └── defaults.rs
├── clipboard/               # 剪贴板
│   ├── mod.rs
│   └── manager.rs
└── legacy/                  # 旧代码迁移
    └── ...
```

---

## 🔄 事件系统设计

### 事件类型

```rust
pub enum Event {
    // 输入事件
    Input(InputEvent),

    // 应用事件
    Action(ActionEvent),

    // 系统事件
    System(SystemEvent),

    // 网络事件
    Network(NetworkEvent),

    // 定时器事件
    Timer(TimerEvent),
}

pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(Rect),
}

pub enum ActionEvent {
    Navigate(Route),
    ShowDialog(DialogType),
    ExecuteCommand(String),
    // ... 更多应用动作
}

pub enum SystemEvent {
    Tick,
    Signal(Signal),
    FocusGained,
    FocusLost,
}
```

### 事件流程

```
用户输入 → 事件分发器 → Action 匹配 → Reducer 处理 → 状态更新
    ↓
重新渲染 → 事件监听 → 循环继续
```

### Reducer 模式

```rust
pub trait Reducer<S, A> {
    fn reduce(&self, state: &mut S, action: A);
}

pub enum Action {
    Navigate(Route),
    GoBack,
    GoForward,
    ShowDialog(DialogType),
    HideDialog,
    ToggleSidebar,
    SwitchTab(usize),
    NewSession,
    SwitchSession(SessionId),
    DeleteSession(SessionId),
    RenameSession(SessionId, String),
    ExecuteCommand(String),
    ShowCommandPalette,
    HideCommandPalette,
    SwitchTheme(String),
    ToggleDarkMode,
    CopySelection,
    Paste,
    // ... 更多动作
}
```

---

## 🧩 组件系统设计

### 基础组件接口

```rust
pub trait Component: Send + Sync {
    // 渲染方法
    fn render(&self, area: Rect, buf: &mut Buffer);

    // 事件处理
    fn handle_event(&mut self, event: &Event) -> ActionResult;

    // 状态查询
    fn get_state(&self) -> ComponentState;

    // 生命周期
    fn on_mount(&mut self);
    fn on_unmount(&mut self);
    fn on_focus(&mut self, focused: bool);
}

pub struct ComponentState {
    pub id: ComponentId,
    pub visible: bool,
    pub focused: bool,
    pub disabled: bool,
    pub bounds: Rect,
}

pub enum ActionResult {
    Handled,
    Ignored,
    Action(Action),
    Actions(Vec<Action>),
}
```

### 组件库分类

#### 1. 布局组件

- **Container**：基础容器组件
- **Flex**：弹性布局
- **Grid**：网格布局
- **Split**：分割布局（可调整）

#### 2. 输入组件

- **TextInput**：单行/多行文本输入
- **Prompt**：命令行提示符
- **CommandPalette**：命令面板 (Ctrl+P)
- **NumberInput**：数字输入

#### 3. 反馈组件

- **Spinner**：加载动画
- **ProgressBar**：进度条
- **Toast**：通知消息
- **Alert**：警告/错误提示

#### 4. 导航组件

- **Sidebar**：侧边栏
- **Tab**：标签页
- **Menu**：菜单
- **Breadcrumb**：面包屑导航

#### 5. 数据组件

- **List**：列表视图
- **Tree**：树形视图
- **Table**：表格视图
- **Editor**：代码编辑器

#### 6. 对话框组件

- **Modal**：模态对话框
- **Confirm**：确认对话框
- **Picker**：选择器对话框

### 组件示例代码

```rust
// Flex 布局组件
pub struct Flex {
    direction: Direction,
    align: Align,
    justify: Justify,
    gap: u16,
    children: Vec<Box<dyn Component>>,
}

impl Component for Flex {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let children_count = self.children.len();
        let total_gap = self.gap * (children_count.saturating_sub(1)) as u16;

        let available_space = match self.direction {
            Direction::Horizontal => area.width - total_gap,
            Direction::Vertical => area.height - total_gap,
        };

        let mut current_pos = match self.direction {
            Direction::Horizontal => area.x,
            Direction::Vertical => area.y,
        };

        for child in &self.children {
            let child_size = available_space / children_count as u16;
            let child_area = match self.direction {
                Direction::Horizontal => Rect {
                    x: current_pos,
                    y: area.y,
                    width: child_size,
                    height: area.height,
                },
                Direction::Vertical => Rect {
                    x: area.x,
                    y: current_pos,
                    width: area.width,
                    height: child_size,
                },
            };

            child.render(child_area, buf);
            current_pos += child_size + self.gap;
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        for child in &mut self.children {
            if let ActionResult::Action(action) = child.handle_event(event) {
                return ActionResult::Action(action);
            }
        }
        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        ComponentState {
            id: ComponentId::new("flex"),
            visible: true,
            focused: false,
            disabled: false,
            bounds: Rect::default(),
        }
    }

    fn on_mount(&mut self) {
        for child in &mut self.children {
            child.on_mount();
        }
    }

    fn on_unmount(&mut self) {
        for child in &mut self.children {
            child.on_unmount();
        }
    }

    fn on_focus(&mut self, focused: bool) {
        // 聚焦第一个子组件
        if !self.children.is_empty() && focused {
            self.children[0].on_focus(true);
        }
    }
}
```

---

## 🎨 主题系统设计

### 主题数据结构

```rust
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub colors: ColorPalette,
    pub styles: StyleSet,
}

pub struct ColorPalette {
    // 主色调
    pub primary: RGB,
    pub secondary: RGB,
    pub accent: RGB,

    // 功能色
    pub success: RGB,
    pub warning: RGB,
    pub error: RGB,
    pub info: RGB,

    // 背景色
    pub bg_primary: RGB,
    pub bg_secondary: RGB,
    pub bg_tertiary: RGB,
    pub bg_input: RGB,

    // 文字色
    pub text_primary: RGB,
    pub text_secondary: RGB,
    pub text_muted: RGB,
    pub text_accent: RGB,

    // 边框色
    pub border: RGB,
    pub border_focus: RGB,
    pub border_active: RGB,

    // 品牌色
    pub brand: RGB,
    pub brand_shimmer: RGB,
}

pub struct StyleSet {
    pub borders: BorderStyle,
    pub fonts: FontStyle,
    pub animations: AnimationConfig,
}
```

### 内置主题

```rust
pub mod themes {
    use super::*;

    pub fn default_dark() -> Theme {
        Theme {
            name: "Default Dark".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: (139, 176, 240),
                secondary: (200, 182, 255),
                accent: (232, 200, 124),
                success: (123, 200, 156),
                warning: (232, 200, 124),
                error: (232, 132, 124),
                info: (139, 176, 240),
                bg_primary: (13, 13, 18),
                bg_secondary: (22, 22, 30),
                bg_tertiary: (30, 30, 46),
                bg_input: (26, 35, 50),
                text_primary: (232, 232, 237),
                text_secondary: (160, 160, 176),
                text_muted: (106, 106, 128),
                text_accent: (200, 182, 255),
                border: (42, 42, 58),
                border_focus: (74, 74, 106),
                border_active: (139, 176, 240),
                brand: (107, 141, 214),
                brand_shimmer: (139, 176, 240),
            },
            styles: StyleSet::default(),
        }
    }

    pub fn default_light() -> Theme {
        // 浅色主题实现
    }

    pub fn minimal() -> Theme {
        // 极简主题实现
    }

    pub fn high_contrast() -> Theme {
        // 高对比度主题实现
    }
}
```

### 主题切换

```rust
// 使用示例
app.dispatch(Action::SwitchTheme("light".to_string()));

// 内部处理
impl Reducer<GlobalState, Action> for ThemeReducer {
    fn reduce(&self, state: &mut GlobalState, action: Action) {
        if let Action::SwitchTheme(name) = action {
            let theme = ThemeRegistry::get(&name);
            state.theme = theme;
            // 重新渲染所有组件
            state.renderer.request_rerender();
        }
    }
}

// 组件中使用主题
impl Component for Button {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let theme = &self.app.state().theme;
        let color = theme.colors.primary;
        let style = Style::default().fg(color);

        Paragraph::new(&self.text)
            .style(style)
            .render(area, buf);
    }
}
```

---

## ⌨️ 快捷键系统设计

### 快捷键绑定结构

```rust
pub struct KeyBinding {
    pub keys: Vec<KeyEvent>,
    pub command: String,
    pub description: String,
    pub mode: KeyMode,
}

pub enum KeyMode {
    Normal,
    Insert,
    Command,
    Visual,
}

pub struct Keymap {
    bindings: HashMap<KeyMode, Vec<KeyBinding>>,
    current_mode: KeyMode,
}

impl Keymap {
    pub fn new() -> Self {
        let mut keymap = Self {
            bindings: HashMap::new(),
            current_mode: KeyMode::Normal,
        };

        // 注册默认快捷键
        keymap.register_defaults();
        keymap
    }

    pub fn register_defaults(&mut self) {
        // 全局快捷键
        self.bind(KeyMode::Normal, KeyBinding {
            keys: vec![KeyEvent::from(KeyCode::Char('p'))],
            command: "command_palette.show".to_string(),
            description: "显示命令面板".to_string(),
            mode: KeyMode::Normal,
        });

        self.bind(KeyMode::Normal, KeyBinding {
            keys: vec![KeyEvent::from(KeyCode::Char('s'))],
            command: "session.save".to_string(),
            description: "保存会话".to_string(),
            mode: KeyMode::Normal,
        });

        // ... 更多快捷键
    }

    pub fn bind(&mut self, mode: KeyMode, binding: KeyBinding) {
        self.bindings.entry(mode)
            .or_insert_with(Vec::new)
            .push(binding);
    }

    pub fn match_key(&self, event: &KeyEvent) -> Option<&KeyBinding> {
        self.bindings.get(&self.current_mode)
            .and_then(|bindings| {
                bindings.iter()
                    .find(|binding| binding.keys.contains(event))
            })
    }

    pub fn switch_mode(&mut self, mode: KeyMode) {
        self.current_mode = mode;
    }
}
```

### 快捷键配置文件

```toml
# ~/.yunxi/keymap.toml

[[normal]]
keys = ["ctrl+p"]
command = "command_palette.show"
description = "显示命令面板"

[[normal]]
keys = ["ctrl+s"]
command = "session.save"
description = "保存会话"

[[normal]]
keys = ["ctrl+w"]
command = "session.close"
description = "关闭会话"

[[insert]]
keys = ["esc"]
command = "mode.normal"
description = "切换到普通模式"

[[command]]
keys = ["esc"]
command = "command_palette.hide"
description = "隐藏命令面板"
```

---

## 🔌 插件系统设计

### Plugin API

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn on_load(&mut self, app: &mut App);
    fn on_unload(&mut self, app: &mut App);

    fn register_commands(&self) -> Vec<Command>;
    fn register_components(&self) -> Vec<Box<dyn Component>>;
    fn register_keybindings(&self) -> Vec<Keybinding>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    commands: CommandRegistry,
    components: ComponentRegistry,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            commands: CommandRegistry::new(),
            components: ComponentRegistry::new(),
        }
    }

    pub fn load(&mut self, mut plugin: Box<dyn Plugin>, app: &mut App) {
        plugin.on_load(app);

        // 注册插件命令
        for command in plugin.register_commands() {
            self.commands.register(command);
        }

        // 注册插件组件
        for component in plugin.register_components() {
            self.components.register(component);
        }

        // 注册快捷键
        for binding in plugin.register_keybindings() {
            app.keymap().bind(binding.mode, binding);
        }

        self.plugins.push(plugin);
    }

    pub fn unload_all(&mut self, app: &mut App) {
        for plugin in &mut self.plugins {
            plugin.on_unload(app);
        }
        self.plugins.clear();
    }
}
```

### 插件示例

```rust
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example_plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn on_load(&mut self, app: &mut App) {
        println!("Plugin loaded: {}", self.name());
    }

    fn on_unload(&mut self, app: &mut App) {
        println!("Plugin unloaded: {}", self.name());
    }

    fn register_commands(&self) -> Vec<Command> {
        vec![
            Command {
                name: "example.hello".to_string(),
                description: "显示示例消息".to_string(),
                handler: Box::new(|app| {
                    app.toast().show("Hello from plugin!");
                }),
            }
        ]
    }

    fn register_components(&self) -> Vec<Box<dyn Component>> {
        vec![
            Box::new(ExampleComponent::new())
        ]
    }

    fn register_keybindings(&self) -> Vec<Keybinding> {
        vec![
            Keybinding {
                keys: vec![KeyEvent::from(KeyCode::Char('e'))],
                command: "example.hello".to_string(),
                description: "显示示例消息".to_string(),
                mode: KeyMode::Normal,
            }
        ]
    }
}
```

---

## 📊 实现阶段规划

### 阶段 1: 核心框架 (2-3 周)

**目标：** 建立新的架构基础

**任务：**
1. ✅ 设计并实现新的目录结构
2. ✅ 实现事件系统核心
3. ✅ 实现状态管理系统
4. ✅ 实现路由系统
5. ✅ 实现生命周期管理
6. ✅ 迁移现有代码到新架构
7. ✅ 建立测试框架

**交付物：**
- 完整的核心框架代码
- 基础事件和状态管理系统
- 路由系统实现
- 现有功能在新架构下的运行

### 阶段 2: 组件库 (2-3 周)

**目标：** 建立完整的组件库

**任务：**
1. 🔨 基础组件
   - Button
   - Input (单行/多行)
   - Label
   - Spacer

2. 🔨 布局组件
   - Flex
   - Grid
   - Split
   - Container

3. 🔨 反馈组件
   - Spinner (多种样式)
   - ProgressBar (线性/圆形)
   - Toast (多种级别)
   - Alert (警告/错误/成功)

4. 🔨 导航组件
   - Sidebar (可折叠)
   - Tab (多标签)
   - Menu (下拉菜单)
   - Breadcrumb

5. 🔨 数据组件
   - List (支持选择)
   - Tree (文件树)
   - Table (表格展示)
   - Editor (代码编辑)

6. 🔨 对话框组件
   - Modal (基础模态框)
   - Confirm (确认对话框)
   - Picker (选择器)
   - InputDialog (输入对话框)

**交付物：**
- 完整的组件库代码
- 组件文档和示例
- 组件测试套件

### 阶段 3: 高级功能 (3-4 周)

**目标：** 实现所有缺失功能

**任务：**
1. 🚧 命令面板
   - Ctrl+P 快捷键
   - 命令搜索
   - 命令历史
   - 命令补全

2. 🚧 完整快捷键系统
   - 快捷键注册
   - 快捷键冲突解决
   - 快捷键配置文件
   - 快捷键帮助界面

3. 🚧 多主题支持
   - 主题系统实现
   - 主题切换界面
   - 主题定制功能
   - 主题导入导出

4. 🚧 插件系统 UI
   - 插件管理界面
   - 插件启用/禁用
   - 插件配置界面
   - 插件市场 (可选)

5. 🚧 文件差异可视化
   - git diff 解析
   - 差异可视化
   - 差异应用/撤销
   - 差异导出

6. 🚧 多选复制功能
   - 文本选择模式
   - 多选复制
   - 选择历史
   - 剪贴板管理

7. 🚧 工作区管理界面
   - 工作区切换
   - 文件树视图
   - 工作区设置
   - 工作区导入导出

8. 🚧 代码语法高亮
   - 语言识别
   - 语法着色
   - 代码块美化
   - 折叠代码块

**交付物：**
- 所有高级功能实现
- 功能测试套件
- 用户文档

### 阶段 4: 优化和测试 (1-2 周)

**目标：** 性能优化和质量保证

**任务：**
1. ⏳ 性能优化
   - 渲染性能优化
   - 内存使用优化
   - 事件处理优化
   - 启动时间优化

2. ⏳ 响应速度优化
   - 输入响应优化
   - 滚动性能优化
   - 动画流畅度优化
   - 大数据量处理优化

3. ⏳ 完整测试覆盖
   - 单元测试
   - 集成测试
   - 端到端测试
   - 性能测试

4. ⏳ 文档完善
   - API 文档
   - 用户指南
   - 开发者文档
   - 迁移指南

**交付物：**
- 性能优化报告
- 完整测试套件
- 完整文档

---

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_layout_horizontal() {
        let mut flex = Flex::new()
            .direction(Direction::Horizontal)
            .gap(1);

        flex.add_child(Box::new(MockComponent::new()));
        flex.add_child(Box::new(MockComponent::new()));

        // 测试布局计算
        let area = Rect::new(0, 0, 20, 10);
        flex.calculate_layout(area);

        // 验证子组件位置
        assert_eq!(flex.children[0].bounds(), Rect::new(0, 0, 9, 10));
        assert_eq!(flex.children[1].bounds(), Rect::new(10, 0, 9, 10));
    }

    #[test]
    fn test_event_propagation() {
        let mut container = Container::new();
        let mut button = Button::new("Click me");

        container.add_child(Box::new(button));

        let event = Event::Input(InputEvent::Key(
            KeyEvent::from(KeyCode::Enter)
        ));

        let result = container.handle_event(&event);
        assert!(matches!(result, ActionResult::Action(_)));
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_command_palette_workflow() {
    let mut app = App::new();
    app.run();

    // 模拟 Ctrl+P
    app.send_event(Event::Input(InputEvent::Key(
        KeyEvent::from(KeyCode::Char('p')).with_modifiers(KeyModifiers::CONTROL)
    )));

    // 等待命令面板显示
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 验证命令面板可见
    assert!(app.find_component("command_palette").is_some());
    assert!(app.find_component("command_palette").unwrap().is_visible());

    // 模拟输入命令
    app.send_event(Event::Input(InputEvent::Key(KeyEvent::from(KeyCode::Char('s')))));
    app.send_event(Event::Input(InputEvent::Key(KeyEvent::from(KeyCode::Char('a')))));
    app.send_event(Event::Input(InputEvent::Key(KeyEvent::from(KeyCode::Char('v')))));
    app.send_event(Event::Input(InputEvent::Key(KeyEvent::from(KeyCode::Enter)))));

    // 验证命令执行
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert!(app.session().is_saved());
}
```

### 性能测试

```rust
#[test]
fn test_render_performance() {
    let mut app = App::new();

    // 创建大量消息
    for _ in 0..1000 {
        app.add_message(Message::user("Test message with some content"));
    }

    // 测量渲染时间
    let start = std::time::Instant::now();
    app.render();
    let duration = start.elapsed();

    // 验证渲染时间 < 100ms
    assert!(duration.as_millis() < 100, "Render took too long: {:?}", duration);
}

#[test]
fn test_memory_usage() {
    let mut app = App::new();

    let initial_memory = get_memory_usage();

    // 添加大量数据
    for _ in 0..10000 {
        app.add_message(Message::user("Test message".repeat(100)));
    }

    let final_memory = get_memory_usage();
    let memory_increase = final_memory - initial_memory;

    // 验证内存增长合理 (< 50MB)
    assert!(memory_increase < 50 * 1024 * 1024, "Memory usage too high: {} MB", memory_increase / 1024 / 1024);
}
```

---

## 📁 配置文件设计

### TUI 配置

```toml
# ~/.yunxi/tui.toml

[general]
# 主题设置
theme = "default_dark"
auto_theme = true  # 根据终端自动选择

# 布局设置
show_sidebar = true
sidebar_width = 30
show_tool_panel = true
tool_panel_width = 35

# 性能设置
max_fps = 60
enable_animations = true
animation_duration_ms = 200

[editor]
# 编辑器设置
tab_size = 4
show_line_numbers = true
word_wrap = false
syntax_highlighting = true

[session]
# 会话设置
auto_save = true
auto_save_interval_seconds = 30
max_history = 1000
confirm_close = true

[keymap]
# 快捷键配置文件路径
config_file = "~/.yunxi/keymap.toml"
mode = "normal"  # normal, insert, command, visual

[clipboard]
# 剪贴板设置
enable = true
max_history = 50
auto_copy = false

[plugins]
# 插件设置
enabled = true
auto_load = true
plugin_directory = "~/.yunxi/plugins"
```

### 快捷键配置

```toml
# ~/.yunxi/keymap.toml

[normal]
# 全局快捷键
[keys = ["ctrl+p"]
command = "command_palette.show"
description = "显示命令面板"

[keys = ["ctrl+s"]
command = "session.save"
description = "保存会话"

[keys = ["ctrl+w"]
command = "session.close"
description = "关闭会话"

[keys = ["ctrl+q"]
command = "app.quit"
description = "退出应用"

[keys = ["ctrl+r"]
command = "session.refresh"
description = "刷新会话"

[keys = ["ctrl+t"]
command = "theme.toggle"
description = "切换主题"

[keys = ["ctrl+b"]
command = "sidebar.toggle"
description = "切换侧边栏"

[keys = ["ctrl+e"]
command = "editor.focus"
description = "聚焦编辑器"

[keys = ["ctrl+k"]
command = "command_palette.history"
description = "命令历史"

[keys = ["ctrl+h"]
command = "help.show"
description = "显示帮助"

[keys = ["ctrl+,"]
command = "settings.open"
description = "打开设置"

[keys = ["f1"]
command = "help.show"
description = "显示帮助"

[keys = ["f2"]
command = "session.rename"
description = "重命名会话"

[keys = ["f5"]
command = "session.refresh"
description = "刷新会话"

[keys = ["f10"]
command = "app.menu"
description = "显示菜单"

[insert]
# 插入模式快捷键
[keys = ["esc"]
command = "mode.normal"
description = "切换到普通模式"

[keys = ["ctrl+c"]
command = "editor.copy"
description = "复制选择"

[keys = ["ctrl+v"]
command = "editor.paste"
description = "粘贴"

[keys = ["ctrl+x"]
command = "editor.cut"
description = "剪切选择"

[keys = ["ctrl+z"]
command = "editor.undo"
description = "撤销"

[keys = ["ctrl+y"]
command = "editor.redo"
description = "重做"

[command]
# 命令模式快捷键
[keys = ["esc"]
command = "command_palette.hide"
description = "隐藏命令面板"

[keys = ["enter"]
command = "command_palette.execute"
description = "执行命令"

[keys = ["ctrl+n"]
command = "command_palette.next"
description = "下一个命令"

[keys = ["ctrl+p"]
command = "command_palette.previous"
description = "上一个命令"

[visual]
# 可视模式快捷键
[keys = ["esc"]
command = "mode.normal"
description = "切换到普通模式"

[keys = ["v"]
command = "selection.toggle"
description = "切换选择"

[keys = ["y"]
command = "selection.copy"
description = "复制选择"

[keys = ["d"]
command = "selection.delete"
description = "删除选择"
```

---

## 🎯 成功标准

### 功能完整性

- ✅ 所有 16 项功能完全实现
- ✅ 与 opencode 功能对等
- ✅ 无功能缺失或降级

### 视觉一致性

- ✅ 像素级视觉复刻
- ✅ 布局结构完全一致
- ✅ 颜色方案完全一致
- ✅ 交互体验完全一致

### 性能要求

- ✅ 启动时间 < 1 秒
- ✅ 渲染帧率 ≥ 60 FPS
- ✅ 输入响应时间 < 50ms
- ✅ 内存使用 < 100MB (正常使用)

### 质量标准

- ✅ 测试覆盖率 ≥ 80%
- ✅ 无已知严重 bug
- ✅ 代码通过所有 lint 检查
- ✅ 文档完整准确

### 可维护性

- ✅ 代码结构清晰
- ✅ 组件可复用性高
- ✅ 易于扩展新功能
- ✅ 易于主题定制

---

## 🚨 风险评估

### 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| ratatui 性能限制 | 中 | 高 | 性能优化、渲染缓存 |
| Rust 生态组件不足 | 中 | 中 | 自研核心组件 |
| 内存泄漏风险 | 低 | 高 | 内存管理、测试覆盖 |
| 终端兼容性问题 | 中 | 中 | 多终端测试、降级方案 |

### 项目风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 时间估算不准确 | 高 | 中 | 分阶段交付、灵活调整 |
| 需求变更 | 中 | 高 | 需求确认、变更管理 |
| 测试覆盖不足 | 中 | 高 | 测试优先、持续测试 |
| 用户体验不达预期 | 低 | 高 | 用户体验测试、快速迭代 |

---

## 📅 里程碑和时间表

### 里程碑 1: 核心框架完成 (Week 3)
- ✅ 新架构建立
- ✅ 事件和状态系统工作
- ✅ 现有功能在新架构下运行

### 里程碑 2: 组件库完成 (Week 6)
- ✅ 所有基础组件实现
- ✅ 组件测试通过
- ✅ 组件文档完成

### 里程碑 3: 高级功能完成 (Week 10)
- ✅ 所有 16 项功能实现
- ✅ 功能测试通过
- ✅ 用户文档完成

### 里程碑 4: 项目交付 (Week 12)
- ✅ 性能优化完成
- ✅ 所有测试通过
- ✅ 文档完善
- ✅ 代码审查通过

---

## 📚 参考资料

### Opencode 源码
- `/Users/xujian/Downloads/opencode-1.15.13/packages/opencode/src/cli/cmd/tui/`
- React/Solid TUI 框架参考
- 组件设计模式参考

### Rust 生态
- [ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI 框架
- [crossterm](https://github.com/crossterm-rs/crossterm) - 终端操作库
- [tui](https://github.com/fdehau/tui-rs) - 早期 TUI 库 (参考)

### 设计模式
- [Flux 架构](https://facebook.github.io/flux/) - 单向数据流
- [Redux 模式](https://redux.js.org/) - 状态管理
- [组件化设计](https://react.dev/learn/thinking-in-react) - 组件思维

---

## ✅ 审核清单

### 设计阶段
- [x] 功能需求分析完成
- [x] 架构设计完成
- [x] 组件设计完成
- [x] 事件系统设计完成
- [x] 主题系统设计完成
- [x] 插件系统设计完成
- [x] 实现阶段规划完成
- [x] 测试策略完成
- [x] 风险评估完成
- [x] 时间表完成

### 实现阶段 (待执行)
- [ ] 核心框架实现
- [ ] 组件库实现
- [ ] 高级功能实现
- [ ] 性能优化
- [ ] 测试覆盖
- [ ] 文档完善

---

**文档状态：** ✅ 已完成
**下一步：** 等待用户确认后，开始执行实现计划