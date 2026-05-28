# YunXi 提示词工程全面重构设计文档

> **版本**: v1.0
> **日期**: 2026-05-29
> **状态**: 待实施

---

## 一、背景与动机

### 1.1 问题现状

2026-05-29 完成的全面审查发现 YunXi 智能体提示词系统存在以下系统性缺陷：

| 类别 | 问题数 | 严重程度 |
|------|--------|----------|
| 准确性错误（引用不一致、法条编号错误、占位符未填充） | 6 | P0 |
| 严重冗余（ASCII 框线、伪代码、重复元数据） | 10+ | P1 |
| Agent 角色提示词过于简略（9个角色仅1句话） | 9 | P1 |
| 缺乏逐步暴露机制（单文件全部注入，无按需加载） | 全量 | P1 |
| 格式不一致（Markdown/YAML/伪代码混杂） | 全量 | P2 |

### 1.2 改造目标

- 提示词平均长度从 450+ 行压缩至 100-150 行（压缩率 75-80%）
- Agent 角色从 1 句话扩充为结构化方法论（信息密度提升 20x）
- 消除所有准确性错误和占位符
- 引入 XML 结构化 + 公共模块复用 + 逐步暴露机制
- 配套 `prompt.rs` / `skill.rs` 运行时支持

---

## 二、新架构设计

### 2.1 四层提示词体系

```
Layer 0: 基础注入层（注入到每次对话）
├── SystemPromptBuilder (prompt.rs) → 通用行为规范
├── Athena 能力声明 (system_prompt.rs) → 工具清单 + 能力边界
└── YUNXI.md → 项目记忆

Layer 1: Foundation 层（全局可用，精简版）
├── HITL 协议 → 4条绝对规则 + 5个确认点（50行）
└── _shared/ 公共模块 → 法律推理框架、输出标准、质检清单

Layer 2: Agent 角色层（按需注入，每会话选1个）
├── assets/agents/retriever.xml
├── assets/agents/analyzer.xml
├── assets/agents/writer.xml
├── assets/agents/novelty_checker.xml
├── assets/agents/creativity_checker.xml
├── assets/agents/infringement_checker.xml
├── assets/agents/invalidity_checker.xml
├── assets/agents/reviewer.xml
└── assets/agents/quality_checker.xml

Layer 3: Skill 能力/任务层（通过 Skill 工具按需加载）
├── 10 个能力层 SKILL（XML 格式，60-150 行）
├── 11 个业务任务 SKILL（XML 格式，80-200 行）
└── 2 个外部服务 SKILL（保持 Markdown，技术文档性质）
```

### 2.2 文件格式规范

| 层 | 格式 | 说明 |
|------|------|------|
| Agent 角色 | .xml | 独立文件，运行时加载 |
| Skill 能力/任务 | .xml | 保留 .md 外壳作为导航索引（可选） |
| 公共模块 | .xml | `assets/skills/_shared/` 目录 |
| 外部服务 | .md | 技术文档，非 LLM 提示词 |
| 注册表 | .md | 保持 `SKILL_REGISTRY.md` 格式 |

---

## 三、XML 标签体系

### 3.1 Agent 标签

```xml
<agent role="role_id" name="中文名称" version="1.0">
  <identity>角色身份描述（必填，1-3句）</identity>
  <methodology>
    <step n="1" name="步骤名">步骤描述</step>
    <step n="2" name="步骤名">步骤描述</step>
  </methodology>
  <output>输出规范（必填）</output>
  <tools>
    <primary>主要工具列表</primary>
    <secondary>辅助工具列表</secondary>
  </tools>
  <constraints>禁止行为和边缘约束</constraints>
</agent>
```

### 3.2 Skill 标签

```xml
<skill id="skill_id" layer="capability|task|service" version="2.0">
  <include ref="_shared/module_name" />
  <identity>能力/任务描述（必填，1-2句）</identity>
  <workflow>
    <step n="1" name="步骤名">...</step>
  </workflow>
  <output>输出格式（必填）</output>
  <error_handling>
    <case type="异常类型">处理策略</case>
  </error_handling>
  <quality>
    <check>检查项</check>
  </quality>
</skill>
```

### 3.3 公共模块标签

```xml
<module id="module_id">
  <!-- 可被 <include ref="module_id"> 引入 -->
  ...复用内容...
</module>
```

---

## 四、公共模块设计（_shared/）

| 文件 | 用途 | 被引用者 |
|------|------|----------|
| `legal_reasoning.xml` | 三段论推理框架：大前提(法条) → 小前提(事实) → 结论 | 全部能力层 SKILL |
| `hitl_protocol.xml` | HITL 5个强制确认点 + 8条中断指令（精简版） | 全部业务任务 SKILL |
| `output_standards.xml` | 输出格式规范：编号体系、引用格式、术语一致性 | 全部 SKILL |
| `quality_checklist.xml` | 通用质量检查清单模板 | 全部 SKILL |
| `patent_glossary.xml` | 专利法律术语标准词汇表（可选，供 LLM 参考） | 按需 |

---

## 五、冗余度目标

| 层 | 当前平均行数 | 目标行数 | 压缩率 |
|------|-------------|---------|--------|
| 能力层 SKILL | ~460 行 | ~100 行 | 78% |
| 业务任务 SKILL | ~570 行 | ~150 行 | 74% |
| Agent 角色 | 1 行 | ~25 行 | 扩充 |
| Foundation HITL | 245 行 | ~50 行 | 80% |
| stop-slop | 210 行 | ~120 行 | 43% |

---

## 六、运行时改造

### 6.1 skill.rs 改造

- 新增 `resolve_includes(xml: &str) -> Result<String>` 函数
- 支持递归展开 `<include ref="...">` 标签（最大深度 3 层）
- 从 `assets/skills/_shared/` 加载公共模块内容

### 6.2 prompt.rs 改造

- `SystemPromptBuilder` 新增 `append_agent_role(role: AgentRole)` 方法
- 从 `assets/agents/{role_id}.xml` 加载角色定义
- 支持 `__AGENT_PROMPT_DYNAMIC_BOUNDARY__` 标记用于后续动态替换

### 6.3 agent_roles.rs 改造

- 移除硬编码的 1 句话 system_prompt()
- 改为从 XML 文件加载的角色定义
- 保留 `allowed_tools()` 和 `name()` 方法
- 新增 `role_xml_path()` 方法返回文件路径

### 6.4 灰度切换

- 用环境变量 `YUNXI_PROMPT_V2=true` 控制新版激活
- 旧版文件保留在 `assets/skills/`（不变），新版放在 `assets/skills-v2/`
- 稳定后删除旧版并移除 feature flag

---

## 七、实施阶段

### Phase 1: 基础设施层（1天）

**交付物：**
- `assets/skills/_shared/` 目录 + 5 个公共模块 XML
- `rust/crates/tools/src/skill.rs` 新增 `resolve_includes()`
- `rust/crates/runtime/src/prompt.rs` 新增 `append_agent_role()`
- 更新 `SKILL_REGISTRY.md`

**验证：** `cargo test -p tools` 全部通过

### Phase 2: Agent 角色层（1天）

**交付物：**
- `assets/agents/` 目录 + 9 个 XML 角色定义
- `agent_roles.rs` 改造为从 XML 加载
- `system_prompt.rs` 引用新角色格式

**验证：** 每个角色可通过 `Skill` 工具加载

### Phase 3: 能力层技能（1.5天）

**交付物：**
- 10 个能力层 SKILL 由 .md 改造为 .xml
- 文件：cap-retrieval, cap-analysis, cap-writing, cap-disclosure-exam, cap-inventive, cap-clarity-exam, cap-invalid, cap-prior-art-ident, cap-response, cap-formal-exam

**验证：** 每个技能用实际专利问题端到端测试

### Phase 4: 业务任务层技能（1.5天）

**交付物：**
- 11 个业务任务 SKILL 由 .md 改造为 .xml
- 文件：task-understand-disclosure, task-prior-art-search, task-write-specification, task-write-claims, task-write-abstract, task-oa-analysis, task-analyze-rejection, task-response-strategy, task-write-response, task-inventive, task-invalid-strategy

**验证：** 交叉引用一致性扫描 + 端到端对话测试

### Phase 5: Foundation + 清理（0.5天）

**交付物：**
- `foundation-hitl/SKILL.md` 精简为 ~50 行 XML
- `.reasonix/skills/stop-slop.md` 迁移到 `assets/skills/stop-slop/SKILL.xml`
- `YUNXI.md` 更新提示词相关约定
- `patent-retrieval` 和 `legal-world-model` 归类为外部服务文档

**验证：** diff 对比检查清单

### Phase 6: 验证 + 回归测试（1天）

**交付物：**
- 全量 `cargo clippy --workspace --all-targets -- -D warnings` 通过
- 全量 `cargo test --workspace` 通过
- 对照审查报告的 15 个问题点逐一确认
- 10 个端到端场景测试通过

---

## 八、质量标准

### 8.1 硬性指标

| 指标 | 目标 | 验证方式 |
|------|------|----------|
| 单文件行数 | 能力层 ≤150, 任务层 ≤200 | `wc -l` |
| 冗余标记 | 无 ASCII 框线、无伪代码、无占位符 | grep 扫描 |
| 准确性 | 法条编号 100% 正确、工具名 100% 一致 | 交叉引用脚本 |
| 编译通过 | `cargo clippy` 零警告 | CI |

### 8.2 软性指标

| 指标 | 目标 | 验证方式 |
|------|------|----------|
| 信息密度 | 每行承载明确指令 | 人工抽样 review |
| 可复用性 | 公共模块被 ≥3 个 SKILL 引用 | 统计 |
| 逐步暴露 | 基础角色 → 工作方法 → 输出格式的递进结构 | 人工 review |

---

## 九、风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| XML 解析失败导致 LLM 无提示词 | 低 | 高 | 保留旧版文件 + feature flag 灰度切换 |
| 压缩过度导致语义丢失 | 中 | 中 | 每个文件对比审查 + 端到端测试 |
| Agent 角色文件路径不一致 | 低 | 中 | 统一命名规范 + 编译期校验 |
| 公共模块 `include` 循环引用 | 低 | 中 | 最大深度 3 层限制 |
| 旧版 SKILL.md 与新版 SKILL.xml 并存混乱 | 中 | 低 | 新版放入 `skills-v2/` 目录 |

---

## 十、附录

### A. 旧版问题清单（15 项，逐一修复对照）

| # | 问题 | 修复方式 |
|----|------|----------|
| 1 | `cap-analysis` 引用错误的 Task 编号 | Phase 3 修正 |
| 2 | `cap-clarity-exam` 引用错误的 TASK 编号 | Phase 3 修正 |
| 3 | `cap-formal-exam` 法条编号错误（实施细则第五十条→第五十三条） | Phase 3 修正 |
| 4 | `system_prompt.rs` 与 `agent_roles.rs` 工具名不一致 | Phase 1/2 统一 |
| 5 | `{{ skeleton('quality_checklist') }}` 占位符 | Phase 3-5 替换 |
| 6 | `patent-retrieval` 引用不存在的脚本路径 | Phase 5 移除 |
| 7 | YAML frontmatter 中 `description` 与正文首行重复 | Phase 3-5 删除重复 |
| 8 | Agent 角色仅 1 句话 | Phase 2 扩充为结构化 XML |
| 9 | ASCII 框线占用 30-50% 篇幅 | Phase 3-5 全部移除 |
| 10 | 伪代码标注"服务暂未实现"仍占篇幅 | Phase 3-5 删除或改为一句话 |
| 11 | HITL 协议关键规则重复 3 次 | Phase 5 精简 |
| 12 | 输出模板过于详细（如 cap-retrieval 的 100 行模板） | Phase 3 浓缩为关键字段 |
| 13 | 多个文件引用不一致（CAP09 vs cap-response） | Phase 3-4 统一 |
| 14 | 元数据重复（每文件 version/author/date） | Phase 5 统一到注册表 |
| 15 | 缺乏 include/复用机制 | Phase 1 新增 |
