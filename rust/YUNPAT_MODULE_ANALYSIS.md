# YunPat Agent → YunXi 模块引入分析报告

> 分析日期：2026-05-18
> 分析范围：`/Users/xujian/projects/yunpat-agent` 全部 Rust crates + TS packages
> 对比基准：YunXi (`/Users/xujian/projects/YunXi/rust`) 现有 13 crates

---

## 一、两项目架构总览

### YunXi 现有架构（13 crates）

```
┌─────────────────────────────────────────────────────────────┐
│  yunxi-cli          CLI REPL 入口（live_cli.rs）              │
├─────────────────────────────────────────────────────────────┤
│  api                Anthropic API 客户端、SSE、OAuth          │
│  commands           SlashCommand 解析、IntentRouter（3级）    │
│  compat-harness     兼容测试框架                              │
│  execpolicy         执行策略、Bash arity、宪法原则（4-tier）  │
│  llm                多供应商 LLM（OpenRouter/Doubao 预设）    │
│  runtime            MCP stdio、StateStore(SQLite)、prompt     │
│                     session、config、bash、permissions         │
│  secrets            OS keyring + file fallback               │
│  tools              工具系统（bash/agent/web/shell/patent...） │
├─────────────────────────────────────────────────────────────┤
│  patent-domain      领域模型（ClaimSet、InventionUnderstanding）│
│  patent-knowledge   规则引擎、guideline_graph、sqlite_graph   │
│  patent-retrieval   混合检索框架（BM25+Vector+Graph 权重）     │
│  patent-workflow    撰写工作流(9阶段)、OA答复工作流(4阶段)      │
└─────────────────────────────────────────────────────────────┘
```

### YunPat 架构（20+ Rust crates + TS packages）

```
┌─────────────────────────────────────────────────────────────┐
│  tui                终端 UI（~171K 行，最大 crate）            │
│  cli / app-server   CLI facade + Axum HTTP 网关              │
├─────────────────────────────────────────────────────────────┤
│  yunpat-agents      专利 Agent 核心（~10.9K 行）               │
│    ├─ PatentAgent / OrchestrationAgent trait 体系            │
│    ├─ 70+ native_tools（检索/撰写/分析/质量/法律）            │
│    ├─ flow / flow_executor（声明式编排 + 质量门控）           │
│    ├─ registry / orchestrator（意图识别 + 路由）              │
│    ├─ memory（四层记忆：HOT/WARM/COLD/ETERNAL）               │
│    ├─ vector_store / knowledge（语义索引 + 混合检索）         │
│    ├─ knowledge_graph（Neo4j 客户端 + Cypher）               │
│    ├─ critique（质量批判循环）                                │
│    ├─ examiner_simulator（审查员模拟器，零 LLM）              │
│    ├─ creativity / drafting / oa_response（领域 Agent）       │
│    └─ invalidation / reexamination / trademark               │
├─────────────────────────────────────────────────────────────┤
│  yunpat-router      意图路由（4级优先级 + 法律场景识别）        │
│  yunpat-models      ModelProvider（stream/embed/multimodal）  │
│  yunpat-core        Runtime、Job、Session、Turn、Compaction    │
│  yunpat-state       SQLite 持久化 + checkpoint schema         │
│  yunpat-orchestration-kernel  HITL 挂起/恢复 + 版本化检查点   │
│  yunpat-workspace-scan  工作区扫描 + yunpat.md 生成           │
│  protocol / hooks / mcp / tools / execpolicy / secrets       │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、模块对比矩阵

| 能力域 | YunXi 现状 | YunPat 对应 | 差距评估 |
|--------|-----------|-------------|---------|
| **Agent Trait 体系** | 无统一 trait，命令式工具调用 | `PatentAgent` + `OrchestrationAgent` + `NativePatentTool` | ⭐⭐⭐ 大 |
| **Flow 编排引擎** | 状态机工作流（drafting_workflow 9阶段） | 声明式 `OrchestrationFlow` + `FlowEngine` + 质量门控 | ⭐⭐⭐ 大 |
| **原生专利工具** | `patent.rs`（~26K 行，通用专利工具） | 70+ 专用工具（claim_generator、oa_parser、quality_scorer...） | ⭐⭐⭐ 大 |
| **工作区扫描** | 无 | `yunpat-workspace-scan`（文档类型推断 + yunpat.md） | ⭐⭐⭐ 大 |
| **审查员模拟** | 无 | `examiner_simulator`（驳回类型检测 + 论证策略） | ⭐⭐ 中 |
| **OA 纯规则解析** | 无 | `oa_parser`（CN/PCT/US/EP 零 LLM 解析） | ⭐⭐ 中 |
| **四层记忆** | `StateStore`（基础会话持久化） | `MemoryStore`（HOT/WARM/COLD/ETERNAL + 访问计数衰减） | ⭐⭐ 中 |
| **向量存储** | 框架定义，无实现 | `VectorStore`（cosine + hybrid search） | ⭐⭐ 中 |
| **HTTP API** | 仅 CLI | `app-server`（Axum + Thread API + SSE） | ⭐⭐ 中 |
| **意图路由** | `IntentRouter`（3级关键词路由） | `yunpat-router`（4级 + 法律场景识别 + 20+ 意图） | ⭐ 小 |
| **Hook 系统** | `runtime/hooks.rs`（简单 pre/post hook） | `hooks` crate（BidirectionalHook + Webhook + JSONL） | ⭐ 小 |
| **HITL 编排** | 无 | `yunpat-orchestration-kernel`（挂起/恢复 + 检查点） | ⭐⭐ 中 |
| **上下文压缩** | `runtime/compact.rs` | `yunpat-core/compaction.rs`（更完善的规划 + 模型感知） | ⭐ 小 |
| **LLM 多供应商** | `llm` crate（OpenRouter/Doubao 预设） | `yunpat-models`（12+ 供应商 + embed + rerank） | ⭐ 小 |
| **MCP** | `runtime/mcp_stdio.rs` | `crates/mcp` + `tui/mcp.rs`（更完整的生命周期） | ⭐ 小 |
| **知识图谱** | `sqlite_graph.rs`（SQLite 图） | `knowledge_graph`（Neo4j + Cypher） | ⭐⭐ 中 |
| **质量批判** | 无 | `critique`（质量/合规评判器 + 循环） | ⭐⭐ 中 |

---

## 三、可引入模块清单（按优先级排序）

### 🔴 P0 — 高价值 · 低耦合 · 建议优先引入

#### 1. `yunpat-workspace-scan` → 新 crate `workspace-scan`

| 属性 | 说明 |
|------|------|
| **功能** | 扫描工作区目录，按文件名与文本预览推断文档类型，生成 `yunpat.md` 项目索引 |
| **核心价值** | YunXi 完全没有工作区初始化能力；对专利项目（含交底书、OA、检索报告等）自动识别类型并生成上下文摘要 |
| **文件规模** | ~566 行（`lib.rs` + `init_doc.rs` + `project_type.rs`） |
| **外部依赖** | 仅 `ignore = "0.4"`（目录遍历） |
| **引入难度** | ⭐ 低 — 无内部依赖，可直接复制并改名 |
| **适配点** | 将 `yunpat.md` 输出格式改为 YunXi 的 `README.md` 或 `.claude/instructions.md` 格式 |
| **YunXi 落点** | `crates/workspace-scan/` 或作为 `runtime` 子模块 |

#### 2. `examiner_simulator` → 融入 `patent-domain` 或新 `patent-simulation`

| 属性 | 说明 |
|------|------|
| **功能** | 审查员模拟器（规则层）：驳回类型检测、论证策略切换、零 LLM 的 OA 预演 |
| **核心价值** | OA 答复前的预演和质量评估；模拟不同审查员风格（严格字面/宽泛解释/组合分析/事后偏见） |
| **文件规模** | ~655 行 |
| **外部依赖** | `strsim`（Jaro 相似度，YunXi 未引入） |
| **引入难度** | ⭐ 低 — 纯算法零外部 IO |
| **适配点** | `RejectionType` / `ArgumentationStrategy` 枚举与 YunXi `patent-domain::models` 中的 `OfficeAction` 模型对接 |
| **YunXi 落点** | `crates/patent-domain/src/examiner_simulator.rs` |

#### 3. `oa_parser`（native_tools） → 融入 `patent-workflow` 或 `patent-tools`

| 属性 | 说明 |
|------|------|
| **功能** | 纯规则型审查意见解析：驳回理由分类（新颖性/创造性/支持/清晰/范围/单一性/形式）、引用文献提取、权利要求编号提取、严重程度评估、应对建议 |
| **核心价值** | 零 LLM 依赖、可离线运行、支持 CN/PCT/US/EP 四种格式；是 OA 答复工作流的关键前置步骤 |
| **文件规模** | ~872 行（`oa_parser.rs`）+ ~192 行（`oa_parser_helpers.rs`） |
| **外部依赖** | 无（纯正则 + 关键词） |
| **引入难度** | ⭐ 低 |
| **适配点** | 输出结构对接 YunXi `patent-domain::models::OfficeAction` |
| **YunXi 落点** | `crates/patent-workflow/src/oa_parser.rs` 或 `crates/tools/src/patent/oa_parser.rs` |

#### 4. `native_tools` 关键子集 → 新 `patent-tools` crate 或融入现有 `tools::patent`

以下工具可按批次引入，每批独立可运行：

**批次 A：撰写辅助（高价值）**
- `claim_generator.rs`（1,344 行）— 权利要求生成器
- `claim_formality_checker.rs` — 权利要求形式检查
- `abstract_drafter.rs` — 摘要撰写
- `specification_drafter.rs` — 说明书撰写
- `writer_tool.rs` — 通用专利撰写辅助

**批次 B：质量检查（高价值）**
- `quality_scorer.rs`（1,181 行）— 多维度质量评分
- `quality_checker.rs` — 质量检查
- `spec_quality_checker.rs` — 说明书质量检查
- `claim_formality_checker.rs` — 权利要求形式检查
- `subject_matter_checker.rs` — 客体审查
- `unity_checker.rs` — 单一性检查
- `spec_formality_checker.rs` — 说明书形式检查
- `unified_quality.rs` — 统一质量评估

**批次 C：策略与分析（中价值）**
- `strategy_scorer.rs` — 答复策略评分
- `strategy_argument_generator.rs` — 论证段落生成
- `success_predictor.rs`（896 行）— 授权成功概率预测
- `prior_art_analyzer.rs` — 现有技术分析
- `innovation_evaluator.rs` — 创新性评估

**批次 D：对比与理解**
- `patent_compare_tool.rs` — 专利对比
- `comparison_analyzer.rs` — 对比分析
- `comparison_report.rs` — 对比报告生成
- `invention_understanding.rs` — 发明理解结构化
- `drawing_understanding.rs` — 附图理解

| 属性 | 说明 |
|------|------|
| **外部依赖** | 各工具不同，多数依赖 `serde` + `regex`；部分需 LLM（通过 `LlmProvider` trait 注入） |
| **引入难度** | ⭐⭐ 中 — 需要为每个工具剥离 `NativePatentTool` trait 依赖，改为 YunXi 的 `ToolSpec` 体系 |
| **适配策略** | 将 `NativePatentTool::execute(Json) -> Json` 签名适配为 YunXi `tools` crate 的 `ToolSpec` + 独立函数 |
| **YunXi 落点** | 逐批放入 `crates/tools/src/patent/` 子模块 |

---

### 🟡 P1 — 中等价值 · 需要一定架构适配

#### 5. `yunpat-router` → 升级 `commands` crate 的 `IntentRouter`

| 属性 | 说明 |
|------|------|
| **功能** | 4级路由优先级：显式命令 → 上下文关联 → 意图识别 → 通用回退；法律场景识别（Domain/Phase/TaskType） |
| **与 YunXi 差异** | YunXi 已有 `commands/src/intent.rs`（3级关键词路由 + 否定检测），YunPat 多了法律场景上下文和 20+ 意图类型 |
| **文件规模** | ~709 行（`orchestrator.rs`）+ router 模块 |
| **引入策略** | **不建议新建 crate**，建议将 `IntentType` 枚举、`INTENT_RULES` 规则表、`identify_scenario_from_input` 场景识别函数融合到现有 `IntentRouter` |
| **引入难度** | ⭐ 低 |

#### 6. `hooks` → 升级 `runtime::hooks`

| 属性 | 说明 |
|------|------|
| **功能** | HookEvent 枚举（UserMessage/ResponseDelta/ToolLifecycle/JobLifecycle/ApprovalLifecycle）、HookSink trait（stdout/JSONL/Webhook）、BidirectionalHook（双向指令注入） |
| **与 YunXi 差异** | YunXi `runtime/src/hooks.rs` 只有 pre_tool_use / post_tool_use 字符串列表；YunPat 是完整的类型化事件系统 |
| **引入策略** | 将 `HookEvent` 和 `HookSink` 概念融入现有 hooks 模块，扩展事件类型 |
| **引入难度** | ⭐⭐ 中 |

#### 7. `memory`（四层记忆） → 升级 `runtime::state_store`

| 属性 | 说明 |
|------|------|
| **功能** | `MemoryStore`（SQLite）：HOT（RAM 本轮）/ WARM（近期会话摘要）/ COLD（长期归档）/ ETERNAL（不可变核心知识）；支持访问计数、时间衰减、downgrade/evict |
| **与 YunXi 差异** | YunXi `StateStore` 只有 session/checkpoint 基础持久化，无分层记忆策略 |
| **引入策略** | 在 `StateStore` 上扩展 `MemoryStore` 子模块，复用现有 SQLite 连接 |
| **引入难度** | ⭐⭐ 中 |
| **YunXi 落点** | `crates/runtime/src/memory.rs` |

#### 8. `vector_store` + `knowledge/index_store` → 升级 `patent-knowledge`

| 属性 | 说明 |
|------|------|
| **功能** | `VectorStore`（内存 HashMap + cosine similarity + hybrid search）+ `SemanticIndexStore`（SQLite chunk + embedding blob） |
| **与 YunXi 差异** | YunXi `patent-retrieval` 只有混合检索的框架定义（权重配置），无实际向量存储和语义索引实现 |
| **引入策略** | 将 `VectorStore` 作为 `patent-knowledge` 的检索后端实现；`SemanticIndexStore` 可作为可选 SQLite 扩展 |
| **引入难度** | ⭐⭐ 中 |

#### 9. `app-server` → 新 crate `app-server`

| 属性 | 说明 |
|------|------|
| **功能** | Axum HTTP 服务器：/healthz、/thread（POST）、/app（POST）、JSON-RPC 适配、Thread 管理、Config 读写 |
| **核心价值** | YunXi 只有 CLI REPL，无 HTTP API；引入后可支持 Web UI、第三方集成 |
| **文件规模** | ~779 行 |
| **外部依赖** | `axum`、`tower-http`、`tokio` |
| **引入难度** | ⭐⭐ 中 — 需要将 YunPat 的 `protocol` 类型（Thread/EventFrame）映射到 YunXi 的 `api`/`runtime` 类型 |
| **YunXi 落点** | `crates/app-server/` |

#### 10. `protocol` → 参考设计而非直接引入

| 属性 | 说明 |
|------|------|
| **功能** | App-server 协议帧：`Thread`、`EventFrame`、`PromptRequest`/`PromptResponse`、`ThreadRequest`/`ThreadResponse` |
| **引入策略** | 不建议直接复制，建议参考其结构设计 YunXi 自己的 `api` 请求/响应类型，确保前后端协议一致性 |
| **引入难度** | ⭐⭐ 中 |

---

### 🟢 P2 — 高价值但高耦合 · 需要架构级重构

#### 11. `yunpat-orchestration-kernel` → 新 crate `orchestration`

| 属性 | 说明 |
|------|------|
| **功能** | `OrchestrationKernel`：运行句柄、步骤执行、HITL（人在回路）挂起/恢复、`HitlPort` trait、`CheckpointEnvelope` 版本化检查点 |
| **核心价值** | 将 YunXi 的静态工作流升级为支持 HITL 的动态编排；用户可在任何步骤暂停、修改、恢复 |
| **文件规模** | ~200 行（骨架，具体业务留白） |
| **引入难度** | ⭐⭐⭐ 高 — 需要重构 `patent-workflow` 的状态机为 `OrchestrationKernel` 驱动的步骤模型 |
| **依赖** | `async-trait`、`tokio`、`uuid` |
| **建议时机** | Phase 4+，等 `patent-workflow` 稳定后再引入 |

#### 12. `Flow` 编排引擎 → 重构 `patent-workflow`

| 属性 | 说明 |
|------|------|
| **功能** | `OrchestrationFlow`（声明式 YAML/JSON 定义）+ `FlowEngine`（条件分支、质量门控、循环、输出累积） |
| **核心价值** | 将硬编码的 9 阶段撰写工作流改为可配置的声明式 Flow；支持质量检查失败自动重试、人工升级 |
| **与 YunXi 差异** | YunXi `drafting_workflow.rs` 是硬编码状态机；YunPat 是通用 Flow 引擎驱动领域 Agent |
| **引入难度** | ⭐⭐⭐ 高 — 需要：① 定义 Flow 声明式格式 ② 实现 FlowEngine ③ 将现有工作流改写为 Flow 定义 |
| **建议时机** | Phase 4+，作为专利工作流 2.0 |

#### 13. `PatentAgent` / `NativePatentTool` trait 体系 → 架构升级

| 属性 | 说明 |
|------|------|
| **功能** | `PatentAgent`（流式 `Stream<StageOutput>`）+ `OrchestrationAgent`（多步 Flow）+ `NativePatentTool`（`async fn execute(Json) -> Json`） |
| **核心价值** | 统一 Agent 和 Tool 的接口；支持 Agent 注册表、动态发现、能力声明 |
| **与 YunXi 差异** | YunXi 工具是命令式函数（`mvp_tool_specs()` 返回静态列表），无 Agent 概念 |
| **引入难度** | ⭐⭐⭐ 高 — 需要将 `tools/src/patent.rs` 重构为 `NativePatentTool` 实现，将 `patent-workflow` 重构为 `PatentAgent` 实现 |
| **建议时机** | 长期架构演进目标，非短期优先 |

#### 14. `knowledge_graph`（Neo4j） → 可选扩展

| 属性 | 说明 |
|------|------|
| **功能** | Neo4j 客户端 + Cypher 查询模板 + Schema 定义（Statute/Case/Ruling/Concept/Article） |
| **核心价值** | 法条关联查询、先例发现、法条适用性分析 |
| **与 YunXi 差异** | YunXi `patent-knowledge` 使用 SQLite 图（`sqlite_graph.rs`），YunPat 使用 Neo4j |
| **引入难度** | ⭐⭐ 中 — 需要 Neo4j 实例；可将 Cypher 模板和 Schema 设计借鉴到 SQLite 图实现中 |
| **建议** | 先借鉴 schema 设计，升级 `sqlite_graph.rs`；Neo4j 客户端作为可选后端 |

#### 15. `critique`（质量批判循环） → 可选引入

| 属性 | 说明 |
|------|------|
| **功能** | `CritiqueLoopResult`、质量评判器（`quality_judge.rs`）、合规评判器（`compliance_judge.rs`）、批判循环执行器 |
| **核心价值** | 输出质量自动评估、合规检查（专利法/审查指南）、迭代改进 |
| **引入难度** | ⭐⭐ 中 |
| **建议时机** | 等 P0/P1 工具稳定后引入，作为质量保障层 |

---

## 四、不建议引入的模块

| 模块 | 理由 |
|------|------|
| `tui` | ~171K 行，过重；YunXi 已有自己的 CLI 架构（`yunxi-cli` + `runtime` REPL） |
| `yunpat-models` | 与 YunXi `llm` crate 功能高度重叠；建议只借鉴 `ModelProvider` trait 设计，不引入代码 |
| `yunpat-mcp-bridge` | ADR-0005 已废弃专利桥接；YunXi `runtime::mcp_stdio` 已覆盖核心 MCP 能力 |
| `core` 中的 `compaction` / `capacity` | YunXi `runtime::compact.rs` 已有类似功能；YunPat 的更完善但差异不大，升级性价比低 |
| `config` | YunXi `runtime::config.rs` 已覆盖配置加载和合并 |
| `state` | YunXi `runtime::state_store.rs` + `session.rs` 已覆盖持久化 |
| TS `packages/*` | 除非需要 Node.js 生态，否则不建议引入；YunXi 是纯 Rust 项目 |

---

## 五、引入实施路线图

```
Phase 1（立即）— 独立工具引入
├── workspace-scan      新 crate，零依赖
├── examiner_simulator  融入 patent-domain
├── oa_parser           融入 patent-workflow
└── vector_store        融入 patent-knowledge

Phase 2（短期）— 工具扩展
├── patent-tools 批次 A（撰写辅助：claim_generator, abstract_drafter...）
├── patent-tools 批次 B（质量检查：quality_scorer, formality_checkers...）
└── IntentRouter 升级（融入 yunpat-router 的法律场景识别）

Phase 3（中期）— 基础设施增强
├── memory 模块（升级 StateStore → 四层记忆）
├── hooks 升级（类型化事件系统）
├── app-server（Axum HTTP API）
└── patent-tools 批次 C/D（策略分析、对比理解）

Phase 4（长期）— 架构升级
├── orchestration-kernel（HITL + 检查点）
├── Flow 引擎（声明式工作流）
├── PatentAgent trait 体系
└── knowledge_graph Neo4j 后端（可选）
```

---

## 六、关键风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| **Rust Edition 差异** | YunPat 使用 2024 edition（`let_chains`），YunXi 使用 2021 | 2024 语法在 2021 中不可用，需手动改写为 `if let` / `match` |
| **依赖膨胀** | 引入大量 crate 导致编译时间增加 | 按需分批引入，使用 feature gate 控制 |
| **trait 体系冲突** | YunPat `NativePatentTool` 与 YunXi `ToolSpec` 签名不同 | 设计适配层（wrapper/bridge），而非直接替换 |
| **测试覆盖** | YunPat 工具可能缺少测试 | 引入时同步编写单元测试，确保 clippy-clean |
| **数据格式不兼容** | YunPat 的 `AgentInput` / `StageOutput` 与 YunXi 模型不同 | 引入时定义转换函数，保持向后兼容 |

---

## 七、总结

**最值得立即引入的 TOP 5 模块：**

1. **`workspace-scan`** — 零依赖、独立可用、填补工作区初始化空白
2. **`examiner_simulator`** — 纯规则零 LLM、OA 预演高价值、低耦合
3. **`oa_parser`** — 零依赖、OA 工作流关键前置、支持四国格式
4. **`claim_generator` + `quality_scorer`** — 专利核心业务工具、可独立运行
5. **`vector_store` + `SemanticIndexStore`** — 填补混合检索的实现空白

**最大架构差距：**

YunXi 目前缺少 **统一的 Agent/Tool trait 体系** 和 **声明式 Flow 编排引擎**。这是 YunPat 相比 YunXi 最核心的架构优势。建议以 P0/P1 工具引入为先导，在 Phase 4 逐步将命令式工作流升级为声明式编排，最终实现与 YunPat 同级别的专利智能体操作系统。
