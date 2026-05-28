# YunXi 智能模型路由器设计文档

> **版本**: v1.0
> **日期**: 2026-05-29
> **状态**: 待实施

---

## 一、背景与动机

### 1.1 问题现状

YunXi 智能体当前支持 `deepseek-v4-pro` 和 `deepseek-v4-flash` 两个模型，但缺乏自动选择机制。用户需要手动指定模型，无法根据任务复杂度动态调整，导致：
- 简单任务使用昂贵的 pro 模型，成本浪费
- 复杂任务使用 flash 模型，质量不足
- 用户体验不流畅，需频繁切换模型

### 1.2 改造目标

实现智能模型路由器，根据任务复杂度自动选择最合适的模型：
- **规划、计划、评估、反思** 等复杂任务 → `deepseek-v4-pro`
- **日常聊天、执行、修改** 等简单任务 → `deepseek-v4-flash`
- 通过配置文件触发 `auto` 模式，支持命令行强制覆盖
- 采用多维度评分机制，平衡质量和成本

---

## 二、整体架构设计

### 2.1 架构层次

在现有 YunXi Rust workspace 中新增独立的 `model-router` crate，作为基础组件被 `yunxi-cli` 和 `runtime` 使用。

```
┌─────────────────┐
│   CLI Layer     │  (yunxi-cli)
│  (--model auto) │
└────────┬────────┘
         │
┌────────▼────────┐
│  Router Layer   │  (model-router)
│  (评分与决策)    │
└────────┬────────┘
         │
┌────────▼────────┐
│  Config Layer   │  (runtime)
│ (策略配置)      │
└────────┬────────┘
         │
┌────────▼────────┐
│ Execution Layer │  (现有)
│  (API 调用)     │
└─────────────────┘
```

### 2.2 设计原则

- **独立隔离**：路由逻辑独立于业务逻辑，易于测试和维护
- **配置驱动**：评分规则可配置，无需重编译即可调优
- **优先级明确**：命令行参数 > 配置文件 > 默认策略
- **优雅降级**：异常时自动回退到 `deepseek-v4-pro`

---

## 三、核心组件设计

### 3.1 `model-router` crate 结构

```
rust/crates/model-router/
├── Cargo.toml
└── src/
    ├── lib.rs              # 对外接口
    ├── selector.rs         # ModelSelector 主选择器
    ├── analyzer.rs         # TaskAnalyzer 任务分析器
    ├── scorer.rs           # ComplexityScorer 复杂度评分器
    ├── config.rs           # ConfigProvider 配置提供器
    ├── types.rs            # 数据类型定义
    └── tests/              # 单元测试
```

### 3.2 组件职责

**1. `ModelSelector`（主选择器）**
```rust
pub struct ModelSelector {
    analyzer: TaskAnalyzer,
    scorer: ComplexityScorer,
    config: RouterConfig,
}

impl ModelSelector {
    /// 选择合适的模型
    pub fn select_model(&self, ctx: &TaskContext) -> Result<ModelSelection>;

    /// 强制覆盖模式
    pub fn select_model_forced(&self, model: &str) -> ModelSelection;
}
```

**2. `TaskAnalyzer`（任务分析器）**
```rust
pub struct TaskAnalyzer;

impl TaskAnalyzer {
    /// 分析用户输入，提取任务特征
    pub fn analyze(&self, input: &UserInput) -> TaskFeatures;
}
```

**3. `ComplexityScorer`（复杂度评分器）**
```rust
pub struct ComplexityScorer {
    config: ScoringConfig,
}

impl ComplexityScorer {
    /// 计算综合复杂度分数（0-100）
    pub fn score(&self, features: &TaskFeatures) -> ComplexityScore;
}
```

**4. `ConfigProvider`（配置提供器）**
```rust
pub struct ConfigProvider;

impl ConfigProvider {
    /// 从 runtime 配置加载路由器配置
    pub fn load(runtime_config: &RuntimeConfig) -> RouterConfig;

    /// 提供默认配置
    pub fn default() -> RouterConfig;
}
```

### 3.3 核心数据结构

**任务特征：**
```rust
pub struct TaskFeatures {
    pub task_type: TaskType,           // 任务类型
    pub input_length: usize,           // 输入长度
    pub has_code: bool,                // 是否包含代码
    pub has_structured_data: bool,     // 是否包含结构化数据
    pub history_rounds: usize,         // 历史对话轮数
    pub files_involved: usize,         // 涉及文件数量
    pub estimated_tool_calls: usize,   // 预估工具调用次数
    pub complex_tools_used: Vec<String>, // 涉及的复杂工具
}
```

**复杂度分数：**
```rust
pub struct ComplexityScore {
    pub total: u8,              // 总分（0-100）
    pub task_type_score: u8,    // 任务类型分数
    pub input_score: u8,        // 输入复杂度分数
    pub context_score: u8,      // 上下文分数
    pub tools_score: u8,        // 工具调用分数
}
```

**模型选择结果：**
```rust
pub struct ModelSelection {
    pub model: String,          // 选定的模型名称
    pub score: ComplexityScore, // 评分详情
    pub reason: String,         // 决策原因
    pub forced: bool,           // 是否为强制覆盖
}
```

---

## 四、评分机制设计

### 4.1 评分维度

总分范围：0-100，阈值：65 分

| 维度 | 权重 | 评分规则 |
|------|------|----------|
| **任务类型** | 40 | 根据关键词匹配，5类任务 |
| **输入复杂度** | 20 | 输入长度 + 代码/数据标记 |
| **上下文复杂度** | 20 | 对话历史 + 文件数量 |
| **工具调用** | 20 | 预估调用次数 + 复杂工具 |

### 4.2 详细评分规则

**1. 任务类型维度（40分）**

```rust
pub enum TaskType {
    Planning,   // 规划类：30-40分
    Analysis,   // 分析类：25-35分
    Generation, // 生成类：20-30分
    Execution,  // 执行类：10-20分
    Chat,       // 聊天类：0-15分
    Unknown,    // 未知：0分
}
```

**关键词映射：**
- 规划类：`["规划", "计划", "设计", "评估", "反思", "策略", "方案"]`
- 分析类：`["分析", "检查", "验证", "审查", "对比", "诊断"]`
- 生成类：`["生成", "撰写", "创建", "起草", "编写", "构建"]`
- 执行类：`["执行", "修改", "操作", "处理", "运行", "应用"]`
- 聊天类：`["聊天", "对话", "解释", "说明", "帮助", "咨询"]`

**2. 输入复杂度维度（20分）**

```rust
let length_score = min(input_length / 100, 10);  // 每100字符1分，上限10分
let code_bonus = if has_code { 5 } else { 0 };
let data_bonus = if has_structured_data { 5 } else { 0 };
let total = length_score + code_bonus + data_bonus;
```

**3. 上下文复杂度维度（20分）**

```rust
let history_score = min(history_rounds, 10);  // 每轮对话1分，上限10分
let files_score = min(files_involved, 10);    // 每个文件2分，上限10分
let total = history_score + files_score;
```

**4. 工具调用维度（20分）**

```rust
let base_score = min(estimated_tool_calls, 15);  // 每次1分，上限15分
let complex_bonus = complex_tools_used.len() as u8 * 5;  // 每个复杂工具+5分
let total = min(base_score + complex_bonus, 20);
```

**复杂工具列表：**
`["python", "bash", "search", "web_fetch", "agent"]`

### 4.3 决策规则

```rust
pub fn decide_model(score: &ComplexityScore) -> &str {
    if score.total >= 65 {
        "deepseek-v4-pro"
    } else {
        "deepseek-v4-flash"
    }
}
```

---

## 五、数据流设计

### 5.1 启动阶段

```
用户执行: yunxi --model auto
    ↓
CLI 解析: parse_args() 识别 model="auto"
    ↓
传递给 Runtime: RuntimeConfig.model = "auto"
    ↓
加载路由器配置: ConfigProvider::load()
    ↓
初始化 ModelSelector
```

### 5.2 请求处理阶段

```
用户输入 + 对话历史 + 上下文
    ↓
TaskAnalyzer::analyze() → TaskFeatures
    ├─ 提取关键词匹配任务类型
    ├─ 计算输入长度和特征
    ├─ 统计历史轮数和文件
    └─ 预估工具调用情况
    ↓
ComplexityScorer::score() → ComplexityScore
    ├─ 计算任务类型分数（40%）
    ├─ 计算输入复杂度分数（20%）
    ├─ 计算上下文复杂度分数（20%）
    └─ 计算工具调用分数（20%）
    ↓
ModelSelector::select_model() → ModelSelection
    ├─ 应用阈值规则（≥65→pro, <65→flash）
    ├─ 生成决策原因
    └─ 返回最终选择
    ↓
使用选定模型进行 API 调用
```

### 5.3 异常处理流程

```
评分失败/配置错误
    ↓
记录 WARN 日志
    ↓
回退到 fallbackModel (deepseek-v4-pro)
    ↓
继续执行，记录决策日志
```

### 5.4 集成点

**CLI 层：**
- `rust/crates/yunxi-cli/src/cli_action.rs:parse_args()`
  - 识别 `--model auto` 参数
  - 解析模型别名

**Runtime 层：**
- `rust/crates/runtime/src/config.rs`
  - 扩展支持 `modelRouter` 配置字段

**执行层：**
- `rust/crates/yunxi-cli/src/live_cli.rs`
  - 每次请求前调用路由器
  - 传递上下文信息

---

## 六、配置设计

### 6.1 配置文件结构

在 `settings.json` 中新增 `modelRouter` 字段：

```json
{
  "modelRouter": {
    "enabled": true,
    "strategy": "balanced",
    "threshold": 65,
    "fallbackModel": "deepseek-v4-pro",
    "dimensions": {
      "taskType": {
        "weight": 40,
        "keywords": {
          "planning": ["规划", "计划", "设计", "评估", "反思", "策略", "方案"],
          "analysis": ["分析", "检查", "验证", "审查", "对比", "诊断"],
          "generation": ["生成", "撰写", "创建", "起草", "编写", "构建"],
          "execution": ["执行", "修改", "操作", "处理", "运行", "应用"],
          "chat": ["聊天", "对话", "解释", "说明", "帮助", "咨询"]
        }
      },
      "inputComplexity": {
        "weight": 20,
        "maxLengthScore": 1000
      },
      "context": {
        "weight": 20,
        "maxHistoryRounds": 10,
        "maxFilesInvolved": 5
      },
      "toolCalls": {
        "weight": 20,
        "complexTools": ["python", "bash", "search", "web_fetch", "agent"]
      }
    },
    "logging": {
      "enabled": true,
      "level": "debug"
    }
  }
}
```

### 6.2 配置字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `enabled` | boolean | 是否启用路由器 |
| `strategy` | string | 策略名称（balanced/cost-optimized/quality-optimized） |
| `threshold` | number | 阈值（0-100） |
| `fallbackModel` | string | 回退模型 |
| `dimensions.*.weight` | number | 维度权重 |
| `logging.enabled` | boolean | 是否启用详细日志 |
| `logging.level` | string | 日志级别（info/debug/warn/error） |

### 6.3 默认配置

当配置文件中未提供 `modelRouter` 时，使用内置默认策略：

```rust
impl RouterConfig {
    pub fn default() -> Self {
        Self {
            enabled: true,
            threshold: 65,
            fallback_model: "deepseek-v4-pro".to_string(),
            task_type_weight: 40,
            input_weight: 20,
            context_weight: 20,
            tools_weight: 20,
            // ... 默认关键词等
        }
    }
}
```

### 6.4 配置加载优先级

1. 项目级：`.yunxi/settings.json`
2. 本地级：`.yunxi/settings.local.json`
3. 用户级：`~/.yunxi/settings.json`
4. 兼容级：`.yunxi.json`（向后兼容）
5. 内置默认值

---

## 七、错误处理

### 7.1 错误场景与处理

| 错误场景 | 处理策略 | 日志级别 |
|----------|----------|----------|
| 配置解析失败 | 使用默认配置，记录警告 | WARN |
| 评分计算异常 | 回退到 fallbackModel，记录原因 | ERROR |
| 无效模型名称 | CLI 解析阶段报错，提示正确选项 | ERROR |
| JSON 格式错误 | 跳过配置加载，使用默认值 | WARN |
| 关键词配置缺失 | 使用内置关键词，记录提示 | INFO |

### 7.2 错误类型定义

```rust
pub enum RouterError {
    ConfigLoadError(String),
    ParseError(String),
    ScoreError(String),
    InvalidModel(String),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::ConfigLoadError(msg) => write!(f, "配置加载失败: {}", msg),
            RouterError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            RouterError::ScoreError(msg) => write!(f, "评分错误: {}", msg),
            RouterError::InvalidModel(msg) => write!(f, "无效模型: {}", msg),
        }
    }
}
```

### 7.3 回退机制

```rust
impl ModelSelector {
    pub fn select_model_safe(&self, ctx: &TaskContext) -> ModelSelection {
        match self.select_model(ctx) {
            Ok(selection) => selection,
            Err(e) => {
                log::warn!("模型选择失败，回退到回退模型: {}", e);
                ModelSelection {
                    model: self.config.fallback_model.clone(),
                    score: ComplexityScore::zero(),
                    reason: format!("回退模式: {}", e),
                    forced: false,
                }
            }
        }
    }
}
```

---

## 八、测试策略

### 8.1 单元测试

**`analyzer.rs` 测试：**
```rust
#[test]
fn test_keyword_matching_planning() {
    let analyzer = TaskAnalyzer::new();
    let features = analyzer.analyze(&UserInput::new("帮我规划一下这个项目的架构"));
    assert_eq!(features.task_type, TaskType::Planning);
}

#[test]
fn test_input_length_scoring() {
    let analyzer = TaskAnalyzer::new();
    let long_input = "a".repeat(2000);
    let features = analyzer.analyze(&UserInput::new(&long_input));
    assert_eq!(features.input_length, 2000);
}
```

**`scorer.rs` 测试：**
```rust
#[test]
fn test_scoring_boundary() {
    let scorer = ComplexityScorer::default();
    let features = TaskFeatures {
        task_type: TaskType::Planning,
        input_length: 1000,
        history_rounds: 10,
        files_involved: 5,
        // ...
    };
    let score = scorer.score(&features);
    assert!(score.total >= 65);
}
```

**`selector.rs` 测试：**
```rust
#[test]
fn test_threshold_decision() {
    let selector = ModelSelector::default();
    // 64分 → flash
    let features_low = create_features_with_score(64);
    let selection_low = selector.select_model(&features_low).unwrap();
    assert_eq!(selection_low.model, "deepseek-v4-flash");

    // 65分 → pro
    let features_high = create_features_with_score(65);
    let selection_high = selector.select_model(&features_high).unwrap();
    assert_eq!(selection_high.model, "deepseek-v4-pro");
}
```

### 8.2 集成测试

**CLI 集成测试：**
```rust
#[test]
fn test_auto_mode_parsing() {
    let args = vec!["--model".to_string(), "auto".to_string()];
    let cli = parse_args(&args).unwrap();
    assert_eq!(cli.model, "auto");
}
```

**端到端测试：**
```rust
#[test]
fn test_e2e_simple_task() {
    let selector = ModelSelector::default();
    let ctx = create_simple_context("帮我修改这个文件");
    let selection = selector.select_model(&ctx).unwrap();
    assert_eq!(selection.model, "deepseek-v4-flash");
}

#[test]
fn test_e2e_complex_task() {
    let selector = ModelSelector::default();
    let ctx = create_complex_context("帮我规划这个大型项目的架构和实施计划");
    let selection = selector.select_model(&ctx).unwrap();
    assert_eq!(selection.model, "deepseek-v4-pro");
}
```

### 8.3 性能测试

```rust
#[test]
fn benchmark_scoring() {
    let selector = ModelSelector::default();
    let ctx = create_typical_context();

    let start = Instant::now();
    for _ in 0..1000 {
        selector.select_model(&ctx).unwrap();
    }
    let duration = start.elapsed();

    assert!(duration.as_millis() < 10_000, "评分耗时应在10ms以内");
}
```

### 8.4 测试覆盖率目标

- `model-router` crate：≥ 80%
- CLI 集成：关键路径 100%
- 整体：≥ 75%

---

## 九、实施阶段

### Phase 1: 基础设施（1天）

**目标：** 创建 `model-router` crate 和基础结构

**交付物：**
- `rust/crates/model-router/Cargo.toml`
- `src/lib.rs`、`src/types.rs`
- 基础数据结构定义
- 单元测试框架

**验证：** `cargo test -p model-router` 通过

---

### Phase 2: 核心逻辑（1.5天）

**目标：** 实现评分和选择核心算法

**交付物：**
- `src/analyzer.rs`（任务分析器）
- `src/scorer.rs`（复杂度评分器）
- `src/selector.rs`（模型选择器）
- 完整单元测试

**验证：**
- 评分算法正确性测试
- 阈值边界测试
- 回退机制测试

---

### Phase 3: 配置集成（0.5天）

**目标：** 扩展配置系统支持路由器配置

**交付物：**
- `src/config.rs`（配置提供器）
- `runtime/src/config.rs` 扩展
- 配置解析逻辑
- 默认策略实现

**验证：**
- 配置文件加载测试
- 配置优先级测试
- 默认配置回退测试

---

### Phase 4: CLI 集成（1天）

**目标：** 集成到现有 CLI 流程

**交付物：**
- `cli_action.rs` 支持 `auto` 模式
- `live_cli.rs` 调用路由器
- 强制覆盖逻辑
- 日志记录

**验证：**
- `--model auto` 参数测试
- 命令行覆盖测试
- 端到端集成测试

---

### Phase 5: 测试与优化（1天）

**目标：** 完善测试并优化性能

**交付物：**
- 集成测试套件
- 性能基准测试
- 性能优化
- 文档完善

**验证：**
- 测试覆盖率 ≥ 80%
- 评分耗时 < 10ms
- `cargo clippy` 零警告

---

### Phase 6: 验证与发布（0.5天）

**目标：** 最终验证和发布准备

**交付物：**
- 全量测试通过
- 代码审查完成
- 文档更新
- 合并到主分支

**验证：**
- 所有测试通过
- 代码审查通过
- 文档完整准确

---

## 十、风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 评分规则不精准导致误判 | 中 | 中 | 配置化权重，支持调优；添加决策日志便于调试 |
| 性能开销影响用户体验 | 低 | 高 | 轻量级分析，目标 <10ms；考虑缓存机制 |
| 与现有配置冲突 | 低 | 中 | 向后兼容，仅在显式设置 `model: "auto"` 时启用 |
| 回退到 pro 导致成本增加 | 中 | 低 | 提供详细的成本追踪；用户可手动关闭路由器 |
| 关键词匹配失败率高 | 中 | 中 | 持续优化关键词库；支持自定义关键词 |

---

## 十一、成功标准

### 11.1 功能指标

- ✅ `--model auto` 正确触发自动选择
- ✅ 命令行参数可强制覆盖（`--model deepseek-v4-pro`）
- ✅ 配置文件支持完整评分策略
- ✅ 回退机制工作正常

### 11.2 质量指标

- ✅ 单元测试覆盖率 ≥ 80%
- ✅ 集成测试通过率 100%
- ✅ `cargo clippy` 零警告
- ✅ `cargo fmt` 格式规范

### 11.3 性能指标

- ✅ 评分耗时 < 10ms（单次请求）
- ✅ 不增加启动延迟
- ✅ 内存占用 < 5MB

### 11.4 用户体验

- ✅ 错误提示清晰友好
- ✅ 日志便于调试（DEBUG 模式）
- ✅ 文档完整准确
- ✅ 配置示例齐全

---

## 十二、后续优化方向

1. **机器学习优化：** 收集真实使用数据，训练更精准的复杂度预测模型
2. **成本追踪：** 详细记录各模型的调用次数和成本
3. **动态阈值：** 根据用户反馈自动调整阈值
4. **A/B 测试：** 支持不同策略的对比测试
5. **多模型扩展：** 支持更多模型（如 Claude、GPT 系列）

---

## 附录

### A. 术语表

| 术语 | 说明 |
|------|------|
| Router | 路由器，负责模型选择的组件 |
| Selector | 选择器，执行具体选择逻辑 |
| Analyzer | 分析器，提取任务特征 |
| Scorer | 评分器，计算复杂度分数 |
| Fallback Model | 回退模型，异常时使用的默认模型 |

### B. 参考资料

- YunXi 项目文档：`YUNXI.md`
- 配置系统实现：`rust/crates/runtime/src/config.rs`
- CLI 参数解析：`rust/crates/yunxi-cli/src/cli_action.rs`

### C. 联系人

- 设计负责人：待指定
- 实施负责人：待指定
- 代码审查：待指定