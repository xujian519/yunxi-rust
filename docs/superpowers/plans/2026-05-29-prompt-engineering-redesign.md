# YunXi 提示词工程全面重构 - 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 YunXi 全部提示词从冗余 Markdown 重构为 XML 结构化格式，引入公共模块复用和 Agent 角色独立文件机制，压缩 75% 冗余度。

**Architecture:** XML 标签体系驱动四层提示词（Foundation/Agent/Capability/Task），公共模块通过 `<include ref="...">` 复用，运行时由 `skill.rs` 和 `agent.rs` 加载并展开。

**Tech Stack:** Rust (prompt.rs, skill.rs, agent_roles.rs, agent.rs), XML (新提示词格式), Markdown (注册表)

---

## 文件结构

### 新建文件（34 个）

| 目录 | 文件 | 行数 |
|------|------|------|
| assets/skills/_shared/ | legal_reasoning.xml | 12 |
| assets/skills/_shared/ | hitl_protocol.xml | 25 |
| assets/skills/_shared/ | output_standards.xml | 10 |
| assets/skills/_shared/ | quality_checklist.xml | 10 |
| assets/skills/_shared/ | patent_glossary.xml | 15 |
| assets/agents/ | retriever.xml | 25 |
| assets/agents/ | analyzer.xml | 25 |
| assets/agents/ | writer.xml | 30 |
| assets/agents/ | novelty_checker.xml | 20 |
| assets/agents/ | creativity_checker.xml | 25 |
| assets/agents/ | infringement_checker.xml | 25 |
| assets/agents/ | invalidity_checker.xml | 25 |
| assets/agents/ | reviewer.xml | 25 |
| assets/agents/ | quality_checker.xml | 25 |
| assets/skills/cap-retrieval/ | SKILL.xml | 50 |
| assets/skills/cap-analysis/ | SKILL.xml | 50 |
| assets/skills/cap-writing/ | SKILL.xml | 60 |
| assets/skills/cap-disclosure-exam/ | SKILL.xml | 50 |
| assets/skills/cap-inventive/ | SKILL.xml | 70 |
| assets/skills/cap-clarity-exam/ | SKILL.xml | 55 |
| assets/skills/cap-invalid/ | SKILL.xml | 55 |
| assets/skills/cap-prior-art-ident/ | SKILL.xml | 50 |
| assets/skills/cap-response/ | SKILL.xml | 55 |
| assets/skills/cap-formal-exam/ | SKILL.xml | 50 |
| assets/skills/ | 11个业务任务 SKILL.xml (每个 50-70行) | ~660 |
| assets/skills/foundation-hitl/ | SKILL.xml | 40 |
| assets/skills/stop-slop/ | SKILL.xml | 50 |

### 修改文件（6 个）

| 文件 | 变更内容 |
|------|----------|
| `rust/crates/tools/src/skill.rs` | 新增 `resolve_includes()` + 修改 `execute_skill()` |
| `rust/crates/runtime/src/prompt.rs` | 新增 `append_agent_role()` 方法 |
| `rust/crates/tools/src/agent_roles.rs` | `system_prompt()` 改为从 XML 文件加载 |
| `rust/crates/tools/src/agent.rs` | `build_agent_system_prompt()` 使用新角色格式 |
| `assets/skills/SKILL_REGISTRY.md` | 更新为 v2.0 格式，新增 Agent/Shared 表 |
| `YUNXI.md` | 新增提示词工程约定 |

## Phase 1: 基础设施层 (1天)

### Task 1.1: 创建 _shared/ 公共模块目录和文件

**Files:**
- Create: `assets/skills/_shared/legal_reasoning.xml`
- Create: `assets/skills/_shared/hitl_protocol.xml`
- Create: `assets/skills/_shared/output_standards.xml`
- Create: `assets/skills/_shared/quality_checklist.xml`
- Create: `assets/skills/_shared/patent_glossary.xml`

- [ ] **Step 1: 创建目录**

Run: `mkdir -p assets/skills/_shared`

- [ ] **Step 2: 创建 legal_reasoning.xml**

```xml
<module id="legal_reasoning">
  <reasoning_framework>
    <step n="1">识别法律问题 — 从用户查询中提取争议焦点和法律要点</step>
    <step n="2">引用相关法条 — 精确到条、款、项，标注来源</step>
    <step n="3">分析技术事实 — 将权利要求逐特征与对比文件比对</step>
    <step n="4">适用法律到事实 — 判断法条规定的构成要件是否满足</step>
    <step n="5">得出结论并标注置信度 — 明确结论，标注 high/medium/low</step>
  </reasoning_framework>
</module>
```

- [ ] **Step 3: 创建 hitl_protocol.xml**

```xml
<module id="hitl_protocol">
  <absolute_rules>
    <rule n="1">用户拥有最终决策权 — 云熙只提供建议</rule>
    <rule n="2">强制确认点不可跳过</rule>
    <rule n="3">未经确认不得执行关键步骤</rule>
    <rule n="4">支持随时中断</rule>
  </absolute_rules>
  <confirmation_points>
    <point n="1" trigger="任务启动前">确认理解、执行计划</point>
    <point n="2" trigger="中间结果">确认方向，是否调整</point>
    <point n="3" trigger="策略选择时">2-3个选项，推荐理由</point>
    <point n="4" trigger="发现风险时">风险描述、等级、缓解</point>
    <point n="5" trigger="任务完成时">是否符合预期</point>
  </confirmation_points>
  <oa_required>
    <point n="1">事实认定 — 审查意见要点准确理解</point>
    <point n="2">法律依据 — 法条和审查指南段落正确</point>
    <point n="3">答复策略 — 各权利要求策略方向</point>
    <point n="4">修改方案 — A33合规性验证</point>
    <point n="5">最终答复 — 逻辑链/证据/格式完整</point>
  </oa_required>
</module>
```

- [ ] **Step 4: 创建 output_standards.xml**

```xml
<module id="output_standards">
  <format>
    <citation>法条引用：《专利法》第XX条第X款，审查指南第X部分第X章第X节</citation>
    <numbering>编号：一、(一)、1.、(1)、①</numbering>
    <doc_ref>对比文件：首次完整信息（CN102241592A，下称D1），后续→D1</doc_ref>
    <terminology>术语一致：同一概念全文统一，不混用同义词</terminology>
  </format>
</module>
```

- [ ] **Step 5: 创建 quality_checklist.xml**

```xml
<module id="quality_checklist">
  <checks>
    <check>所有法条引用精确到条款号</check>
    <check>对比文件引用格式一致</check>
    <check>论点有法条或案例支撑</check>
    <check>结论明确且包含置信度标注</check>
    <check>输出格式符合 output_standards 规范</check>
  </checks>
</module>
```

- [ ] **Step 6: 创建 patent_glossary.xml**

```xml
<module id="patent_glossary">
  <terms>
    <term key="现有技术">申请日以前在国内外为公众所知的技术（A22.5）</term>
    <term key="新颖性">不属于现有技术，也无抵触申请（A22.2）</term>
    <term key="创造性">非显而易见 + 实质性特点 + 显著进步（A22.3）</term>
    <term key="三步法">确定最接近现有技术 → 区别特征和技术问题 → 技术启示判断</term>
    <term key="技术启示">本领域技术人员是否有动机将区别特征应用于最接近现有技术</term>
    <term key="充分公开">说明书清楚完整，能够实现为准（A26.3）</term>
    <term key="清楚性">以说明书为依据，清楚简要限定保护范围（A26.4）</term>
    <term key="单一性">一件申请限于一项发明，同一构思可合案（A31.1）</term>
  </terms>
</module>
```

- [ ] **Step 7: Commit**

```bash
git add assets/skills/_shared/
git commit -m "feat(prompt): 创建 _shared/ 公共提示词模块"

### Task 1.2: 实现 skill.rs 的 XML include 解析

**Files:**
- Modify: `rust/crates/tools/src/skill.rs:1-123`

- [ ] **Step 1: 在 skill.rs 添加 resolve_includes 函数**

在 `parse_skill_description` 函数之后（第117行之后）添加:

```rust
const MAX_INCLUDE_DEPTH: usize = 3;

pub(crate) fn resolve_includes(content: &str, base_dir: &std::path::Path) -> Result<String, String> {
    let mut resolved = content.to_string();
    for _depth in 0..MAX_INCLUDE_DEPTH {
        let mut replaced = false;
        let mut result = String::with_capacity(resolved.len());
        let mut remaining = resolved.as_str();

        while let Some(tag_start) = remaining.find("<include ") {
            result.push_str(&remaining[..tag_start]);
            let after_start = &remaining[tag_start..];
            let (module_content, consumed) = parse_include_tag(after_start, base_dir)?;
            result.push_str(&module_content);
            remaining = &after_start[consumed..];
            replaced = true;
        }
        result.push_str(remaining);
        resolved = result;
        if !replaced {
            break;
        }
    }
    Ok(resolved)
}

fn parse_include_tag(input: &str, base_dir: &std::path::Path) -> Result<(String, usize), String> {
    let ref_start = input.find("ref=\"").ok_or("invalid include tag: missing ref")?;
    let ref_val_start = ref_start + 5;
    let ref_val_end = input[ref_val_start..].find('"')
        .map(|p| ref_val_start + p)
        .ok_or("invalid include tag: unclosed ref")?;
    let ref_value = &input[ref_val_start..ref_val_end];
    let tag_end = input[ref_val_end..].find('>')
        .map(|p| ref_val_end + p)
        .ok_or("invalid include tag: missing >")?;

    let module_path = base_dir.join(ref_value).with_extension("xml");
    let module_content = std::fs::read_to_string(&module_path)
        .map_err(|e| format!("failed to read include '{}': {e}", module_path.display()))?;
    Ok((module_content, tag_end + 1))
}
```

- [ ] **Step 2: 修改 execute_skill 函数以支持 XML**

修改第18-30行的 `execute_skill`:

```rust
pub(crate) fn execute_skill(input: SkillInput) -> Result<SkillOutput, String> {
    let skill_path = resolve_skill_path(&input.skill)?;
    let raw_prompt = std::fs::read_to_string(&skill_path).map_err(|e| e.to_string())?;

    let prompt = if raw_prompt.trim_start().starts_with("<?xml")
        || raw_prompt.trim_start().starts_with("<skill")
    {
        let skills_dir = skill_path.parent().ok_or("invalid skill path")?;
        resolve_includes(&raw_prompt, skills_dir)?
    } else {
        raw_prompt
    };

    let description = parse_skill_description(&prompt);
    Ok(SkillOutput {
        skill: input.skill,
        path: skill_path.display().to_string(),
        args: input.args,
        description,
        prompt,
    })
}
```

- [ ] **Step 3: 运行测试验证**

Run: `cargo test -p tools skill`
Expected: 现有测试全部通过

- [ ] **Step 4: Commit**

```bash
git add rust/crates/tools/src/skill.rs
git commit -m "feat(prompt): 实现 XML include 展开机制"


### Task 1.3: 实现 prompt.rs 的 Agent 角色加载方法

**Files:**
- Modify: `rust/crates/runtime/src/prompt.rs:120-218`

- [ ] **Step 1: 添加 append_agent_role 方法**

在 `prompt.rs` 第162行 `append_section` 之后添加:

```rust
    #[must_use]
    pub fn append_agent_role(mut self, role_xml_path: &Path) -> Self {
        if let Ok(content) = std::fs::read_to_string(role_xml_path) {
            self.append_sections.push(format!(
                "## Agent Role\n\n{content}"
            ));
        }
        self
    }
```

- [ ] **Step 2: 运行测试**

Run: `cargo test -p runtime prompt`
Expected: 全部通过

- [ ] **Step 3: Commit**

```bash
git add rust/crates/runtime/src/prompt.rs
git commit -m "feat(prompt): SystemPromptBuilder 新增 append_agent_role"


### Task 1.4: 更新 SKILL_REGISTRY.md

**Files:**
- Modify: `assets/skills/SKILL_REGISTRY.md`

- [ ] **Step 1: 更新注册表为 v2.0 格式**

在注册表第1行起，替换内容。关键新增:
- 公共模块 (`_shared/`) 表
- Agent 角色 (`assets/agents/`) 表
- 所有文件路径从 .md 改为 .xml

- [ ] **Step 2: Commit**

```bash
git add assets/skills/SKILL_REGISTRY.md
git commit -m "docs(registry): 更新 SKILL_REGISTRY 适配 XML v2.0"


## Phase 2: Agent 角色层 (1天)

### Task 2.1: 创建 9 个 Agent 角色 XML（分别 file）

**Files:**
- Create: `assets/agents/retriever.xml`
- Create: `assets/agents/analyzer.xml`
- Create: `assets/agents/writer.xml`
- Create: `assets/agents/novelty_checker.xml`
- Create: `assets/agents/creativity_checker.xml`
- Create: `assets/agents/infringement_checker.xml`
- Create: `assets/agents/invalidity_checker.xml`
- Create: `assets/agents/reviewer.xml`
- Create: `assets/agents/quality_checker.xml`

每个文件遵循相同 XML 模板:

```xml
<agent role="role_id" name="中文名称" version="1.0">
  <identity>角色身份 1-2 句</identity>
  <methodology>
    <step n="1" name="步骤名">步骤描述</step>
    ...
  </methodology>
  <output>输出规范</output>
  <tools>
    <primary>主要工具逗号分隔</primary>
    <secondary>辅助工具逗号分隔</secondary>
  </tools>
  <constraints>禁止行为和边缘约束</constraints>
</agent>
```

具体内容参照设计文档第二节中的示例（已获用户认可）。

每个文件创建后验证:
```bash
wc -l assets/agents/{role_id}.xml
# Expected: 每次 ≤30 行
```

- [ ] **Step 1-9: 依次创建 9 个文件，每文件 commit**

```bash
git add assets/agents/
git commit -m "feat(prompt): 创建 9 个 Agent 角色 XML 定义"
```

### Task 2.2: 改造 agent_roles.rs 从 XML 加载

**Files:**
- Modify: `rust/crates/tools/src/agent_roles.rs:23-37`

- [ ] **Step 1: 修改 system_prompt() 优先从 XML 加载**

```rust
    pub fn system_prompt(&self) -> String {
        let role_id = self.role_id();
        let cargo_manifest = env!("CARGO_MANIFEST_DIR");
        let xml_path = std::path::PathBuf::from(cargo_manifest)
            .join("../../../assets/agents")
            .join(format!("{role_id}.xml"));

        if let Ok(content) = std::fs::read_to_string(&xml_path) {
            return content;
        }
        // 回退到硬编码
        match self {
            Self::Retriever => "专利检索专家。检索相关专利、现有技术和文献。",
            Self::Analyzer => "专利分析专家。深入分析专利文件，提取关键技术特征。",
            ...
        }.to_string()
    }

    fn role_id(&self) -> &str {
        match self {
            Self::Retriever => "retriever",
            Self::Analyzer => "analyzer",
            ...
        }
    }
```

- [ ] **Step 2: 更新 from_str_opt 添加新的匹配别名**

```rust
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "retriever" => Some(Self::Retriever),
            "analyzer" => Some(Self::Analyzer),
            "writer" => Some(Self::Writer),
            "novelty" | "novelty_checker" => Some(Self::NoveltyChecker),
            "creativity" | "creativity_checker" => Some(Self::CreativityChecker),
            "infringement" | "infringement_checker" => Some(Self::InfringementChecker),
            "invalidity" | "invalidity_checker" => Some(Self::InvalidityChecker),
            "reviewer" => Some(Self::Reviewer),
            "quality" | "quality_checker" => Some(Self::QualityChecker),
            _ => None,
        }
    }
```

- [ ] **Step 3: 运行测试**

Run: `cargo test -p tools agent_roles`
Expected: 全部通过

- [ ] **Step 4: Commit**

```bash
git add rust/crates/tools/src/agent_roles.rs
git commit -m "feat(prompt): agent_roles 改为从 XML 文件加载角色定义"


### Task 2.3: 更新 build_agent_system_prompt

**Files:**
- Modify: `rust/crates/tools/src/agent.rs:351-376`

- [ ] **Step 1: 修改 build_agent_system_prompt 展开 XML include**

```rust
fn build_agent_system_prompt(subagent_type: &str) -> Result<Vec<String>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut prompt = load_system_prompt(cwd, DEFAULT_AGENT_SYSTEM_DATE.to_string(),
        std::env::consts::OS, "unknown").map_err(|e| e.to_string())?;

    crate::system_prompt::append_athena_capabilities(&mut prompt);

    if let Some(role) = crate::agent_roles::AgentRole::from_str_opt(subagent_type) {
        let role_prompt = role.system_prompt();
        let skills_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../assets/skills");
        let resolved = crate::skill::resolve_includes(&role_prompt, &skills_dir)
            .unwrap_or(role_prompt);
        prompt.push(resolved);
    } else {
        prompt.push(format!("You are a background sub-agent of type `{subagent_type}`..."));
    }
    Ok(prompt)
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test -p tools agent`
Expected: 全部通过

- [ ] **Step 3: Commit**

```bash
git add rust/crates/tools/src/agent.rs
git commit -m "feat(prompt): agent system prompt 使用 XML 角色定义"


## Phase 3: 能力层技能 (1.5天)

以下 10 个 Task (3.1-3.10) 将旧版 SKILL.md 改写为 SKILL.xml。每个文件统一模板:

```xml
<skill id="skill_id" layer="capability" version="2.0">
  <include ref="_shared/legal_reasoning" />
  <include ref="_shared/output_standards" />

  <identity>能力描述 1-2 句</identity>

  <workflow>
    <step n="1" name="步骤名">...</step>
  </workflow>

  <output>输出规范和格式</output>

  <error_handling>
    <case type="异常">处理策略</case>
  </error_handling>

  <quality>
    <check>检查项</check>
  </quality>
</skill>
```

每个文件执行流程：
1. 创建 SKILL.xml
2. `wc -l` 验证 ≤60 行
3. 与旧版 SKILL.md 对比检查关键信息不丢失
4. Commit

### Task 3.1: cap-retrieval
- 旧行数: 323 → 目标: 50
- 保留: 5步检索流程（识别关键词→多源检索→图谱补全→法条画像→综合排序）
- 删除: 100行输出模板、3个特殊场景的完整示例、pseudo-code

### Task 3.2: cap-analysis
- 旧行数: 471 → 目标: 50
- 保留: 三级技术分析框架、7维解构、四层对比矩阵
- 删除: 300+行 ASCII 对比表、重复的HITL确认点模板
- 修复: Task 2.2 引用 → task-analyze-rejection

### Task 3.3: cap-writing
- 旧行数: 419 → 目标: 60
- 保留: 3种文书类型结构（无效宣告/审查答复/复审请求）
- 删除: 100行伪代码

### Task 3.4: cap-disclosure-exam
- 旧行数: 418 → 目标: 50
- 保留: A26.3三标准（清楚/完整/可实现）

### Task 3.5: cap-inventive
- 旧行数: 384 → 目标: 70
- 保留: 三步法 + JSON输出格式 + 降级规则

### Task 3.6: cap-clarity-exam
- 旧行数: 528 → 目标: 55
- 修复: TASK_1_4 引用错误 → task-write-claims

### Task 3.7: cap-invalid
- 旧行数: 465 → 目标: 55
- 保留: 5种证据组合策略；删除120行伪代码

### Task 3.8: cap-prior-art-ident
- 旧行数: 575 → 目标: 50
- 保留: 5种特殊情形（抵触申请/保密协议等）

### Task 3.9: cap-response
- 旧行数: 487 → 目标: 55
- 保留: 3种答复策略 + 4种特殊场景

### Task 3.10: cap-formal-exam
- 旧行数: 527 → 目标: 50
- 修复: 《细则》第五十条 → 第五十三条


## Phase 4: 业务任务层技能 (1.5天)

11 个业务任务 SKILL 均按模板压缩。每个约 50-70 行。流程同 Phase 3。

| Task | Skill ID | 旧行数 | 目标 | 关键保留内容 |
|------|----------|--------|------|-------------|
| 4.1 | task-understand-disclosure | 543 | 60 | 5个HITL确认点、创新点提取框架 |
| 4.2 | task-prior-art-search | 879 | 70 | 3种检索策略、数据源清单 |
| 4.3 | task-write-specification | 562 | 60 | 7部分说明书结构 |
| 4.4 | task-write-claims | 540 | 60 | A/B/C布局方案 |
| 4.5 | task-write-abstract | 700 | 50 | 摘要规范、关键词选择 |
| 4.6 | task-oa-analysis | 469 | 60 | 5个Turn并行流程 |
| 4.7 | task-analyze-rejection | 416 | 55 | 3种分析模板 |
| 4.8 | task-response-strategy | 545 | 60 | 3种答复策略 |
| 4.9 | task-write-response | 724 | 70 | 完整答复结构 |
| 4.10 | task-inventive | 376 | 50 | 三步法+综合评估 |
| 4.11 | task-invalid-strategy | 503 | 55 | 证据组合+成功率评估 |


## Phase 5: Foundation + 清理 (0.5天)

### Task 5.1: foundation-hitl 精简为 XML

**Files:**
- Create: `assets/skills/foundation-hitl/SKILL.xml` (约40行，保留4条规则+5个确认点+OA特定5点)

- [ ] **Step 1: 创建**

```bash
wc -l assets/skills/foundation-hitl/SKILL.xml
# Expected: ≤40
```

- [ ] **Step 2: Commit**

```bash
git add assets/skills/foundation-hitl/SKILL.xml
git commit -m "refactor(prompt): foundation-hitl 精简为 XML (245行 → 40行)"


### Task 5.2: stop-slop 迁移

**Files:**
- Create: `assets/skills/stop-slop/SKILL.xml` (约50行，保留R1-R9规则+精简流程)

- [ ] **Step 1: 创建并提交**

```bash
git add assets/skills/stop-slop/SKILL.xml
git commit -m "feat(prompt): stop-slop 从 .reasonix 迁移到 assets/skills/"


### Task 5.3: 更新 YUNXI.md

**Files:**
- Modify: `YUNXI.md`

- [ ] **Step 1: 在第80行之前添加提示词约定**

```markdown
## 提示词约定
- 新建和修改提示词使用 XML 格式（`.xml`），公共模块放在 `assets/skills/_shared/`
- Agent 角色定义放在 `assets/agents/`，每个角色独立一个 XML 文件
- SKILL.xml 中的 `<include ref="_shared/module_name" />` 由 `skill.rs` 展开
- 提示词单文件上限：能力层 ≤150行，任务层 ≤200行
- 禁止使用 ASCII 框线、伪代码、未填充的模板占位符
```

- [ ] **Step 2: Commit**

```bash
git add YUNXI.md
git commit -m "docs: YUNXI.md 新增提示词工程约定"


## Phase 6: 验证 + 回归测试 (1天)

### Task 6.1: 全量编译和测试

- [ ] **Step 1: Cargo clippy**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: 零警告

- [ ] **Step 2: Cargo test**

Run: `cargo test --workspace`
Expected: 全部通过

- [ ] **Step 3: XML 格式验证**

```bash
find assets/skills -name "SKILL.xml" | wc -l
# Expected: 24

find assets/skills -name "*.xml" -exec head -1 {} \; | grep -v '<?xml\|^<skill\|^<agent\|^<module'
# Expected: 零输出（所有文件以合法 XML 或标签开头）
```

- [ ] **Step 4: 零占位符验证**

```bash
grep -rn "\{\{" assets/skills/ assets/agents/
# Expected: 零匹配
```

- [ ] **Step 5: 零ASCII框线验证**

```bash
grep -rn "┌\|└\|├\|│" assets/skills/*.xml assets/agents/*.xml
# Expected: 零匹配
```

- [ ] **Step 6: 零伪代码验证**

```bash
grep -rn "def \|```python" assets/skills/*.xml
# Expected: 零匹配
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "verify: Phase 6 全量验证通过"


### Task 6.2: 15项旧版问题回归检查

每项验证命令 + 预期：

| # | 验证命令 | 预期 |
|---|----------|------|
| 1 | `grep "task-oa-analysis\|task-analyze-rejection" assets/skills/cap-analysis/SKILL.xml` | 匹配新引用 |
| 2 | `grep "task-write-claims" assets/skills/cap-clarity-exam/SKILL.xml` | 匹配新引用 |
| 3 | `grep "第五十" assets/skills/cap-formal-exam/SKILL.xml` | 零匹配 |
| 4 | 对比 `system_prompt.rs` 与 `agent_roles.rs` 工具名 | 一致 |
| 5 | `grep -rn "\{\{" assets/skills/ assets/agents/` | 零匹配 |
| 6 | `grep "patent_search_cli" assets/skills/patent-retrieval/` | 保留于 .md |
| 7 | `find assets/skills -name "SKILL.xml" -exec grep "description:" {} \;` | 0（XML无该问题） |
| 8 | `wc -l assets/agents/*.xml` | 每文件15-30行 |
| 9 | `grep -rn "┌" assets/skills/*.xml` | 零匹配 |
| 10 | `grep -rn "def \|```python" assets/skills/*.xml` | 零匹配 |
| 11 | `grep -c "用户拥有最终决策权" assets/skills/foundation-hitl/SKILL.xml` | 1 |
| 12 | `wc -l assets/skills/cap-retrieval/SKILL.xml` | ≤50 |
| 13 | `grep "CAP09\|CAPABILITY_9" assets/skills/` | 零匹配 |
| 14 | `find assets/skills -name "SKILL.xml" -exec grep "创建时间\|设计者" {} \;` | 0 |
| 15 | `grep -c "<include" assets/skills/*.xml` | ≥15 |

- [ ] **Step: 修复所有失败项，重新验证后提交**

```bash
git commit -m "verify: 15项旧版问题全部修复确认"


### Task 6.3: 最终清理

- [ ] **Step 1: 暂不删除旧版 SKILL.md**
  - 新旧版双轨运行，旧版保留作为回退
  - 稳定2周后统一清理旧版

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "chore: 新旧版双轨完成，进入稳定观察期"


## 附录：实施统计

| 指标 | 数量 |
|------|------|
| 新建 XML 文件 | 34 个 |
| 修改 Rust 源文件 | 4 个 |
| 修改 Markdown 文件 | 2 个 |
| 总预估工时 | 6-8 天 |
| 压缩率（能力层） | 78% |
| 压缩率（任务层） | 74% |
| Agent 角色信息密度提升 | 20x |

---

**计划完成。**
