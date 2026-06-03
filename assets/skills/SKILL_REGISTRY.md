# YunXi 专利智能体 Skill 注册表

## 使用方法

在 REPL 中使用 `Skill` 工具加载：

```
Skill({"skill": "cap-retrieval"})
```

## 公共模块 (_shared/)

| 模块 ID | 描述 | 被引用者 |
|----------|------|----------|
| `_shared/legal_reasoning` | 三段论推理框架 | 全部能力层 SKILL |
| `_shared/hitl_protocol` | HITL 确认点协议 | 全部业务任务 SKILL |
| `_shared/output_standards` | 输出格式规范 | 全部 SKILL |
| `_shared/quality_checklist` | 质量检查清单 | 全部 SKILL |
| `_shared/patent_glossary` | 专利法律术语词汇表 | 按需 |

## Agent 角色 (assets/agents/)

| 角色 ID | 名称 | 文件 |
|----------|------|------|
| `retriever` | 检索专家 | `agents/retriever.xml` |
| `analyzer` | 分析专家 | `agents/analyzer.xml` |
| `writer` | 撰写专家 | `agents/writer.xml` |
| `novelty_checker` | 新颖性评估专家 | `agents/novelty_checker.xml` |
| `creativity_checker` | 创造性评估专家 | `agents/creativity_checker.xml` |
| `infringement_checker` | 侵权分析专家 | `agents/infringement_checker.xml` |
| `invalidity_checker` | 无效分析专家 | `agents/invalidity_checker.xml` |
| `reviewer` | 文件审查专家 | `agents/reviewer.xml` |
| `quality_checker` | 质量评估专家 | `agents/quality_checker.xml` |

## 能力层技能 (Capability)

| Skill ID | 描述 | 文件 |
|----------|------|------|
| `cap-retrieval` | 法律检索能力 | `cap-retrieval/SKILL.xml` |
| `cap-analysis` | 专利分析能力 | `cap-analysis/SKILL.xml` |
| `cap-writing` | 专利撰写能力 | `cap-writing/SKILL.xml` |
| `cap-disclosure-exam` | 交底书审查能力 | `cap-disclosure-exam/SKILL.xml` |
| `cap-inventive` | 创造性判断能力 | `cap-inventive/SKILL.xml` |
| `cap-clarity-exam` | 清楚性审查能力 | `cap-clarity-exam/SKILL.xml` |
| `cap-invalid` | 无效宣告能力 | `cap-invalid/SKILL.xml` |
| `cap-prior-art-ident` | 对比文件识别能力 | `cap-prior-art-ident/SKILL.xml` |
| `cap-response` | 答复策略能力 | `cap-response/SKILL.xml` |
| `cap-formal-exam` | 形式审查能力 | `cap-formal-exam/SKILL.xml` |

## 业务任务技能 (Business Task)

| Skill ID | 描述 | 文件 |
|----------|------|------|
| `task-understand-disclosure` | 理解技术交底书 | `task-understand-disclosure/SKILL.xml` |
| `task-prior-art-search` | 现有技术检索 | `task-prior-art-search/SKILL.xml` |
| `task-write-specification` | 撰写说明书 | `task-write-specification/SKILL.xml` |
| `task-write-claims` | 撰写权利要求 | `task-write-claims/SKILL.xml` |
| `task-write-abstract` | 撰写摘要 | `task-write-abstract/SKILL.xml` |
| `task-oa-analysis` | 审查意见解读 | `task-oa-analysis/SKILL.xml` |
| `task-analyze-rejection` | 驳回理由分析 | `task-analyze-rejection/SKILL.xml` |
| `task-response-strategy` | 答复策略制定 | `task-response-strategy/SKILL.xml` |
| `task-write-response` | 答复文本撰写 | `task-write-response/SKILL.xml` |
| `task-inventive` | 创造性分析任务 | `task-inventive/SKILL.xml` |
| `task-invalid-strategy` | 无效策略任务 | `task-invalid-strategy/SKILL.xml` |

## 基础技能 (Foundation)

| Skill ID | 描述 | 文件 |
|----------|------|------|
| `foundation-hitl` | 人机协作强制协议 | `foundation-hitl/SKILL.xml` |

## 外部服务技能 (Service)

| Skill ID | 描述 | 文件 |
|----------|------|------|
| `patent-retrieval` | PostgreSQL 专利数据库检索 | `patent-retrieval/SKILL.md` |
| `legal-world-model` | 三层法律知识库问答 | `legal-world-model/SKILL.md` |

## 技巧 (Technique)

| Skill ID | 描述 | 文件 |
|----------|------|------|
| `stop-slop` | 专利法律文书去冗余精简 | `stop-slop/SKILL.xml` |
| `hanlin-academy` | 多模型集体审议（翰林院） | `technique/hanlin-academy/SKILL.xml` |

## 数据来源

- 项目内置 skill 定义（`assets/skills/`）
- XML 格式 v2.0，支持 `<include ref="_shared/...">` 模块复用
- 更新时间：2026-05-29