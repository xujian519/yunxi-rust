# YunXi Phase 1-4 优化计划 — 产品需求文档 (PRD)

> 文档版本：v1.0
> 日期：2026-06-04
> 作者：产品经理 Alice
> 项目：YunXi 专业专利 AI Agent

---

## 1. 项目信息

| 字段 | 值 |
|------|-----|
| 项目名称 | `yunxi_phase1_4_optimization` |
| 编程语言 | Rust（Workspace, 18 crates） |
| 项目路径 | `/Users/xujian/projects/YunXi` |
| 语言 | 中文 |

### 原始需求复述

基于 YunXi 项目全面审查报告的发现，分四个 Phase 实施优化：Phase 1 实现反思引擎替换 STUB；Phase 2 建立感知→学习→记忆→推理闭环；Phase 3 实现 MCP 远程传输；Phase 4 完成 P1 级质量改善。共涉及 4 个 P0 问题和 6 个 P1 问题的修复。

---

## 2. 产品目标

| # | 目标 | 度量指标 |
|---|------|---------|
| G1 | **自我纠错能力从零到可用**：LLM 反思器从硬编码 STUB 升级为真实 LLM 驱动的自我纠错引擎，使推理管线具备 Correction 阶段自动修正能力 | 反思器调用 LLM 成功率 ≥ 95%；Correction 阶段输出质量评分提升 ≥ 20% |
| G2 | **闭环运行**：打通感知→学习→记忆→推理四大模块之间的数据断裂，使系统能从历史交互中学习并影响后续决策 | Hebbian 权重实时影响工具推荐；推理结论 100% 写入记忆；意图分类器读取用户偏好 |
| G3 | **远程 MCP 连通性 & 系统质量达标**：SSE/HTTP/WS 传输可用，dispatch 可维护，全链路异步安全，工具输出格式统一 | MCP 远程传输连接成功率 ≥ 99%；dispatch 零 match 分支；无 `block_on()` 嵌套风险；所有工具返回 `ToolOutput` |

---

## 3. 用户故事

### Phase 1：实现反思引擎

| # | 用户故事 |
|---|---------|
| US1.1 | 作为专利分析用户，我希望系统能自动检查推理输出中的逻辑漏洞并修正，这样我得到的法律分析结论更可靠 |
| US1.2 | 作为系统运维人员，我希望反思器在 LLM 不可用时优雅降级而非崩溃，这样系统可用性有保障 |
| US1.3 | 作为专利审查员，我希望宪法引擎能对超出规则引擎能力的复杂合规问题调用 LLM 深度分析，这样审查结果不会遗漏模糊风险 |
| US1.4 | 作为产品经理，我希望反思评估结果写入记忆并驱动推理策略调整，这样系统能持续自改进 |

### Phase 2：感知→学习→记忆→推理闭环

| # | 用户故事 |
|---|---------|
| US2.1 | 作为频繁使用检索+分析组合流程的用户，我希望系统自动学习我的工具使用模式并优先推荐，这样减少我的操作步骤 |
| US2.2 | 作为专业用户，我希望之前推理得出的法律结论在后续会话中可被自动引用，这样不用每次重复分析 |
| US2.3 | 作为新用户，我希望意图分类器能根据我的历史偏好调整分类结果，这样分类更贴合我的使用习惯 |
| US2.4 | 作为数据工程师，我希望记忆系统支持基于语义的相似内容检索，这样即使关键词不同也能找到相关记忆 |

### Phase 3：MCP 远程传输

| # | 用户故事 |
|---|---------|
| US3.1 | 作为企业用户，我希望连接远程 MCP 服务器（SSE/HTTP/WS），这样我能使用部署在内网的专有工具 |
| US3.2 | 作为运维人员，我希望 MCP 长连接在断线后自动重连，这样网络抖动不会中断服务 |
| US3.3 | 作为高并发用户，我希望 MCP 连接使用连接池，这样多工具并发调用不会因创建 Runtime 导致性能瓶颈 |

### Phase 4：P1 质量改善

| # | 用户故事 |
|---|---------|
| US4.1 | 作为使用冷门意图（如防御性撰写、宽保护策略）的用户，我希望这些意图也能被正确识别，这样我不必每次用通用意图代替 |
| US4.2 | 作为工具开发者，我希望新工具通过实现 trait 自动注册而非手动添加 match 分支，这样减少合并冲突和遗漏 |
| US4.3 | 作为系统稳定性工程师，我希望全链路异步化消除 `block_on()` 嵌套风险，这样不会出现运行时 panic |
| US4.4 | 作为下游集成开发者，我希望所有工具输出统一为 `ToolOutput` 结构，这样我的解析代码只需对接一种格式 |
| US4.5 | 作为推理质量优化者，我希望假设去重使用 embedding 相似度，这样语义相同但表述不同的假设能被正确识别为重复 |
| US4.6 | 作为合规审查员，我希望宪法引擎在规则匹配不足时自动调用 LLM 分析，这样复杂合规检查不会因规则遗漏而放行 |

---

## 4. 需求池

### P0 — Must Have（阻塞核心功能）

| ID | Phase | 需求 | 验收标准 | 涉及文件 |
|----|-------|------|---------|---------|
| P0-1 | 1 | 实现 `LLMReflectionReflector`，替换硬编码 STUB | 1. `reflect_on_llm_output()` 调用 `LlmClient` 获取真实反思结果；2. STUB 代码完全移除；3. 单元测试覆盖 LLM 成功/失败/超时场景；4. 原有测试全部通过 | `tools/src/reflection/llm_reflection.rs` |
| P0-2 | 1 | 反思结果集成到推理管线 Correction 阶段 | 1. `ReasoningPipeline::execute()` 在 Correction 阶段调用 `Reflector`；2. 反思驱动的重试逻辑生效（`should_retry=true` 时重新执行）；3. 重试次数不超过 `max_retries=3` | `reasoning/src/pipeline.rs` |
| P0-3 | 1 | 宪法引擎 LLM 增强 | 1. `evaluate_rule()` 中 `_` 分支（无法识别的检查类型）调用 LLM 进行深度分析；2. LLM 分析结果以 `RuleCheckResult` 返回；3. LLM 不可用时降级为当前关键词匹配逻辑 | `constitutional-engine/src/engine.rs` |
| P0-4 | 1 | 评估结果闭环 | 1. 反思/评估结果写入 `UnifiedMemory`；2. 写入的记忆包含评估分数、改进建议、策略调整建议；3. 下次推理时可检索到历史评估结果 | `tools/src/reflection/`, `memory/src/unified.rs` |
| P0-5 | 2 | Hebbian 学习反馈到 WorkflowRouter | 1. `WorkflowRouter::suggest_resources()` 读取 `HebbianOptimizer::find_optimal_path()` 的结果；2. Hebbian 推荐的工具按权重排序插入 `suggested_tools`；3. Strong/Moderate/Weak 对应不同推荐权重 | `router/src/workflow_router.rs`, `memory/src/hebbian.rs` |
| P0-6 | 2 | 推理结论写入 UnifiedMemory | 1. `ReasoningPipeline::execute()` 完成后，将 `final_conclusion` 写入 `UnifiedMemory::remember()`；2. 写入记忆的 key 包含 session_id + 问题摘要；3. 后续会话可通过 `search()` 检索到 | `reasoning/src/pipeline.rs`, `memory/src/unified.rs` |
| P0-7 | 2 | 意图分类器集成记忆系统 | 1. `IntentClassifier::classify()` 从 `UnifiedMemory` 读取用户偏好；2. 用户偏好作为置信度调整因子影响分类结果；3. 偏好记录自动从用户修正行为中提取 | `intent/src/classifier.rs`, `memory/src/unified.rs` |
| P0-8 | 2 | 记忆语义检索 | 1. `UnifiedMemory::search()` 支持基于 embedding 向量的语义搜索；2. 语义搜索与关键词搜索结果合并、去重；3. 搜索结果按相关度排序 | `memory/src/unified.rs`, `embedding/` |
| P0-9 | 3 | 实现 SSE 传输层 | 1. 实现 `SseTransport` struct，符合 `rmcp` 传输协议；2. 支持 SSE 事件流解析和重连；3. 集成到 `McpServerManager`；4. 连接成功/失败有明确状态报告 | `mcp-bridge/`, `runtime/mcp*` |
| P0-10 | 3 | 实现 HTTP 传输层 | 1. 实现 `HttpTransport` struct，支持 JSON-RPC over HTTP；2. 超时和错误处理完备；3. 集成到 `McpServerManager` | `mcp-bridge/`, `runtime/mcp*` |
| P0-11 | 3 | 实现 WebSocket 传输层 | 1. 实现 `WsTransport` struct，支持双向 JSON-RPC over WS；2. 心跳保活机制；3. 集成到 `McpServerManager` | `mcp-bridge/`, `runtime/mcp*` |
| P0-12 | 3 | 断线重连机制 | 1. SSE/WS 连接断开后自动指数退避重连（最大间隔 30s）；2. 重连后恢复工具发现状态；3. 重连失败 3 次后标记为 `disconnected` 并通知上层 | `mcp-bridge/`, `runtime/mcp*` |
| P0-13 | 3 | 连接池优化 | 1. `McpRuntime` 复用全局 `tokio::Runtime` 而非每次 `call_tool()` 创建；2. 连接池大小可配置；3. idle 连接超时回收 | `mcp-bridge/src/lib.rs` |

### P1 — Should Have（提升系统质量）

| ID | Phase | 需求 | 验收标准 | 涉及文件 |
|----|-------|------|---------|---------|
| P1-1 | 4 | `active_intents()` 扩展到 50 种或配置驱动 | 1. 所有 50 种 `IntentType` 均在 `active_intents()` 中激活；或改为从配置文件加载活跃意图列表；2. `classify_by_embedding()` 的标签列表对齐 50 种意图；3. 新增意图无需修改分类器源码 | `intent/src/classifier.rs` |
| P1-2 | 4 | `dispatch.rs` 重构为 trait-based registry | 1. 定义 `Tool` trait（`name()`, `execute()`, `input_schema()`, `output_schema()`）；2. 使用 `inventory` 或 `linkme` 实现编译期自动注册；3. `dispatch.rs` 的 match 语句消除；4. 新增工具只需实现 trait + 注册宏 | `tools/src/dispatch.rs` |
| P1-3 | 4 | 全链路异步化消除 `block_on()` | 1. `tools/lib.rs`、`mcp-bridge/src/lib.rs`、`llm/src/lib.rs` 中所有 `block_on()` 消除；2. 工具执行接口改为 `async fn execute()`；3. 无 tokio 运行时嵌套风险；4. `cargo clippy` 无 `block_on` 相关警告 | `tools/`, `mcp-bridge/`, `llm/` |
| P1-4 | 4 | 专利工具输出格式统一为 `ToolOutput` | 1. 定义 `ToolOutput` struct（`code: ResultCode`, `data: Value`, `error: Option<String>`, `metadata: HashMap`）；2. 所有专利工具 runner 统一返回 `Result<ToolOutput, ToolError>`；3. 现有 prompt 文本格式保持兼容（可通过 `metadata.format` 区分） | `tools/src/patent*.rs`, `tools/src/dispatch.rs` |
| P1-5 | 4 | 假设去重改用 embedding 相似度 | 1. `HypothesisManager::is_duplicate()` 使用 `embedding::cosine_similarity()` 判断；2. 相似度阈值可配置（默认 0.85）；3. embedding 不可用时回退到精确字符串匹配；4. 单元测试覆盖语义相似但表述不同的假设场景 | `reasoning/src/hypothesis.rs` |
| P1-6 | 4 | 宪法引擎 LLM 增强分析 | 1. `ConstitutionalEngine` 在 `evaluate_rule()` 无法识别检查类型时调用 LLM；2. LLM 分析结果解析为 `RuleCheckResult`；3. LLM 增强检查有 `confidence` 和 `latency` 指标；4. LLM 不可用时降级为默认 `confidence=0.5` | `constitutional-engine/src/engine.rs` |

### P2 — Nice to Have（提升系统能力，Phase 4 范围外，仅作记录）

| ID | 需求 | 说明 |
|----|------|------|
| P2-1 | 记忆自动 ETERNAL 晋升 | 基于访问频率和 Hebbian 连接强度自动晋升 |
| P2-2 | 模型路由配置化 | ModelSelector 从配置文件加载策略 |
| P2-3 | Provider base URL 配置化 | 从配置文件加载，支持私有部署 |
| P2-4 | 多轮意图追踪 | 引入 IntentHistory 跨轮次累积 |
| P2-5 | 领域术语外部文件加载 | YAML/JSON 配置替代硬编码 |

---

## 5. UI 设计草案

> 本项目为 Rust 后端系统，无 GUI 界面。以下为 CLI/API 层面的交互设计。

### 5.1 反思引擎（Phase 1）

```
用户查询 → ReasoningPipeline.execute()
  → Phase 1-5 正常执行
  → Phase 6 (Correction):
      → LLMReflectionReflector.reflect()
      → if should_retry && retries < 3: 重新执行低分阶段
      → 反思结果写入 UnifiedMemory
  → 返回最终结论
```

CLI 输出增加反思信息：
```
[reasoning] Correction: 发现 2 个问题，1 个改进建议
[reasoning] 问题: 技术描述不够详细; 缺少具体实施方式
[reasoning] 建议: 增加技术细节描述
[reasoning] 重试: 否，当前输出质量可接受 (score: 82.0)
```

### 5.2 Hebbian 学习反馈（Phase 2）

`WorkflowRouter` 推荐工具时增加 Hebbian 信号：
```
[router] 领域=Patent, 复杂度=Medium, 意图=NOVELTY_APPLICATION
[router] 基础推荐: ClaimParse, KnowledgeGraphQuery, KnowledgeSearch, LegalReasoning, ...
[router] Hebbian 增强: +SemanticCompare (strength=0.72), +NoveltyAnalysis (strength=0.65)
```

### 5.3 MCP 状态报告（Phase 3）

`mcp status` 命令输出扩展：
```
MCP Server Status:
  remote-search    [SSE]     ✅ ready     12 tools
  patent-db        [HTTP]    ✅ ready      8 tools
  analysis-ws      [WS]      ⚠️ reconnecting (attempt 2/3)
  local-tools      [Stdio]   ✅ ready      5 tools
```

---

## 6. 待确认问题

| # | 问题 | 影响范围 | 建议默认方案 |
|---|------|---------|-------------|
| Q1 | **反思器 LLM Provider 选择**：反思调用使用与主推理相同的 Provider/Model 还是独立的轻量模型？ | Phase 1 (P0-1) | 使用独立轻量模型（如 flash），避免反思过程占用主推理资源 |
| Q2 | **Hebbian 推荐权重公式**：Hebbian 连接强度如何与现有规则推荐权重融合？加权平均、乘性增强、还是仅作为排序因子？ | Phase 2 (P0-5) | 乘性增强：`final_score = base_score * (1 + hebbian_strength * 0.5)` |
| Q3 | **记忆语义检索 embedding 模型**：`UnifiedMemory::search()` 的语义检索使用现有的 BGE-M3 还是独立的轻量模型？ | Phase 2 (P0-8) | 复用现有 BGE-M3，共享 embedding 缓存 |
| Q4 | **MCP 传输实现方式**：SSE/HTTP/WS 是基于 `rmcp` crate 扩展还是从零实现？ | Phase 3 (P0-9~11) | 基于 `rmcp` crate 扩展，复用已有的传输 trait 定义 |
| Q5 | **`dispatch.rs` 重构策略**：使用 `inventory` crate、`linkme` crate 还是自定义注册宏？是否有运行时动态注册的需求？ | Phase 4 (P1-2) | 使用 `inventory` crate 编译期注册，暂无运行时动态注册需求 |
| Q6 | **`ToolOutput` 兼容性**：统一 `ToolOutput` 后，现有依赖 `Result<String, String>` 的上层代码如何迁移？是否需要过渡期？ | Phase 4 (P1-4) | `ToolOutput` 实现 `Display` trait输出 JSON 字符串，上层代码可渐进迁移 |
| Q7 | **Phase 间是否需要硬隔离**：Phase 2 依赖 Phase 1 的反思能力吗？Phase 4 能否与 Phase 2 并行开发？ | 全局 | Phase 1 是 Phase 2 的前置依赖（反思结果需写入记忆）；Phase 3 独立于 Phase 1/2；Phase 4 中 P1-2/P1-3 可与 Phase 2 并行，P1-5/P1-6 依赖 Phase 2 的 embedding 集成 |
| Q8 | **假设去重 embedding 相似度阈值**：0.85 的默认阈值是否合适？是否需要根据领域/语言调整？ | Phase 4 (P1-5) | 默认 0.85，可通过配置文件调整，范围 [0.7, 0.95] |
| Q9 | **宪法引擎 LLM 调用频率控制**：每次规则检查都调用 LLM 会显著增加延迟和成本，如何控制调用频率？ | Phase 1 (P0-3) + Phase 4 (P1-6) | 仅对 `_` 分支（无法识别的检查类型）和 `confidence < 0.6` 的结果调用 LLM，结果缓存 1h |
| Q10 | **block_on 消除策略**：全链路异步化涉及 API 层变更，是否需要保持同步 API 兼容层？ | Phase 4 (P1-3) | 提供 `spawn_blocking` 桥接层，标记 `#[deprecated]`，给下游 2 个版本迁移期 |

---

## 附录：Phase 依赖关系

```
Phase 1: 反思引擎
  │
  ├── P0-1 LLM 反思器实现
  ├── P0-2 反思集成到 Correction 阶段 (依赖 P0-1)
  ├── P0-3 宪法引擎 LLM 增强 (独立)
  └── P0-4 评估结果闭环 (依赖 P0-1, P0-2)
         │
Phase 2: 感知→学习→记忆→推理闭环
  │
  ├── P0-5 Hebbian → WorkflowRouter (独立)
  ├── P0-6 推理结论 → 记忆 (独立)
  ├── P0-7 意图分类器 ← 记忆 (依赖 P0-6)
  └── P0-8 记忆语义检索 (依赖 embedding 基础设施)
         │
Phase 3: MCP 远程传输 (独立于 Phase 1/2)
  │
  ├── P0-9 SSE 传输
  ├── P0-10 HTTP 传输
  ├── P0-11 WebSocket 传输
  ├── P0-12 断线重连 (依赖 P0-9~11)
  └── P0-13 连接池 (依赖 P0-9~11)
         │
Phase 4: P1 质量改善
  │
  ├── P1-1 active_intents 扩展 (独立)
  ├── P1-2 dispatch 重构 (独立)
  ├── P1-3 全链路异步化 (独立)
  ├── P1-4 ToolOutput 统一 (独立)
  ├── P1-5 假设去重 embedding (依赖 Phase 2 的 embedding 集成)
  └── P1-6 宪法引擎 LLM 增强 (与 P0-3 合并或复用)
```

---

*本 PRD 基于审查报告和源码分析生成，所有需求均包含可验证的验收标准。*
