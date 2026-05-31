---
name: task-oa-analysis
description: 云熙 L4 业务层提示词 - 任务2_1：审查意见解读与问题分解
version: "1.0"
---
description: 任务2_1：审查意见解读与问题分解 v2.0
# 任务2_1：审查意见解读与问题分解 v2.0

> **场景**: 答复审查意见
> **步骤**: 第1步 - 审查意见解读与问题分解
> **版本**: v2.0-with-parallel-calls
> **创建时间**: 2026-04-19
> **基于**: v1.0
> **新增**: 并行工具调用指令 + Playbook 编排模式
> **设计者**: YunXi Agent v1.0

---

## 任务概述

**任务名称**: 审查意见通知书解读与问题分解

**任务目标**:
- 准确理解审查员的核心观点
- 识别所有驳回理由和问题点
- 为后续分析奠定基础

**输入材料**:
- 审查意见通知书PDF
- 本申请的原始申请文件
- （可选）之前的审查意见和答复历史

**输出成果**:
- 驳回理由清单
- 引用的对比文件列表
- 问题严重程度评估

---

## 执行流程 (带并行调用指令)

### ⚡ Turn 1: 并行读取（所有独立操作同时进行）

> **注意**: 以下为设计参考伪代码，对应服务暂未实现。

```python
# 并行读取所有必要文件
async def read_all_documents_parallel(oa_path, application_number):
    """并行读取所有文档"""

    # 并行执行所有读取操作
    results = await parallel([
        # 读取1: 审查意见通知书PDF
        read_pdf(oa_path),

        # 读取2: 本申请原始申请文件（从数据库）
        query_postgresql("""
            SELECT application_number, filing_date, title,
                   abstract, claims, description
            FROM patents
            WHERE application_number = %s
        """, (application_number,)),

        # 读取3: 之前的审查意见历史（如果有）
        query_office_action_history(application_number),

        # 读取4: 审查指南相关章节（用于后续分析）
        query_legal_knowledge("""
            MATCH (g:GuidanceChapter)
            WHERE g.title CONTAINS "审查意见"
            RETURN g.content
        """)
    ])

    oa_content, application_data, oa_history, guidance_content = results
    return {
        "oa_content": oa_content,
        "application_data": application_data,
        "oa_history": oa_history,
        "guidance_content": guidance_content
    }
```

**🤝 人机交互点1：确认材料完整**

```
【云熙】用户，我已完成并行文档读取。

📊 并行读取结果：
✅ 审查意见通知书PDF: 已读取（12页）
✅ 本申请原始文件: 已从数据库读取
✅ 审查意见历史: 找到2轮历史
✅ 审查指南章节: 已加载相关内容

🤝 请您确认：
- 材料是否齐全？
- 是否有其他背景信息需要补充？

【您可以选择】：
A. 材料齐全，开始分析
B. 补充材料（请提供）
C. 先查看申请信息
```

---

### ⚡ Turn 2: 并行提取（基于Turn 1的结果）

```python
# 并行提取关键信息
async def extract_key_information_parallel(documents):
    """并行提取关键信息"""

    # 并行执行所有提取任务
    results = await parallel([
        # 提取1: 基本信息
        extract_basic_info(documents["oa_content"]),

        # 提取2: 驳回理由
        extract_rejection_reasons(documents["oa_content"]),

        # 提取3: 引用的对比文件
        extract_cited_documents(documents["oa_content"]),

        # 提取4: 审查员意见
        extract_examiner_arguments(documents["oa_content"]),

        # 提取5: 法律依据
        extract_legal_basis(documents["oa_content"])
    ])

    basic_info, rejections, citations, arguments, legal_basis = results
    return {
        "basic_info": basic_info,
        "rejections": rejections,
        "citations": citations,
        "arguments": arguments,
        "legal_basis": legal_basis
    }
```

**输出格式**:
```markdown
【第二步: 并行信息提取完成】

✅ 已完成并行提取：
- 基本信息: 提取完成
- 驳回理由: 识别到3个驳回理由
- 对比文件: 识别到5个对比文件
- 审查员意见: 提取完成
- 法律依据: 识别到5条法律依据

## 提取的基本信息

| 项目 | 内容 |
|-----|------|
| 申请号 | CNXXXXXXXXX.X |
| 申请日 | YYYY-MM-DD |
| 发明名称 | XXXXXX |
| 审查员 | XXX |
| 审查意见日 | YYYY-MM-DD |
| 答复期限 | YYYY-MM-DD |
| 审查轮次 | 一通/二通/... |
```

---

### ⚡ Turn 3: 并行分析（基于Turn 2的结果）

```python
# 并行分析驳回理由
async def analyze_rejections_parallel(extracted_info):
    """并行分析驳回理由"""

    # 为每个驳回理由并行分析
    analysis_tasks = [
        analyze_single_rejection(rejection, extracted_info)
        for rejection in extracted_info["rejections"]
    ]

    results = await parallel(analysis_tasks)
    return results

async def analyze_single_rejection(rejection, context):
    """分析单个驳回理由（内部也是并行调用）"""

    # 并行检索支持信息
    results = await parallel([
        # 检索1: 相似案例
        search_similar_cases(rejection),

        # 检索2: 法律条文解释
        query_legal_interpretation(rejection["legal_basis"]),

        # 检索3: 审查指南相关段落
        query_guidance_paragraphs(rejection["type"])
    ])

    similar_cases, legal_interp, guidance_paras = results

    return {
        "rejection": rejection,
        "similar_cases": similar_cases,
        "legal_interpretation": legal_interp,
        "guidance_paras": guidance_paras,
        "severity": assess_severity(rejection, similar_cases)
    }
```

**🤝 人机交互点2：展示驳回理由清单**

```
【云熙】用户，我已完成审查意见的并行分析。

📊 驳回理由清单（并行分析完成）：

┌─────────────────────────────────────────────────────────┐
│  理由1: 新颖性问题 (权利要求1-3)                           │
│  法条依据: 《专利法》第22条第2款                          │
│  引用对比文件: D1 (CNXXXXXXXXX.U)                         │
│  相似案例: 找到15件相似案例                               │
│  严重程度: ⚠️ 中等                                         │
├─────────────────────────────────────────────────────────┤
│  理由2: 创造性问题 (权利要求4-5)                          │
│  法条依据: 《专利法》第22条第3款                          │
│  引用对比文件: D1+D2                                      │
│  相似案例: 找到23件相似案例                               │
│  严重程度: 🔴 严重                                         │
├─────────────────────────────────────────────────────────┤
│  理由3: 说明书公开不充分 (说明书第{{X}}段)                 │
│  法条依据: 《专利法》第26条第3款                          │
│  相似案例: 找到8件相似案例                                │
│  严重程度: 🔴 严重                                         │
└─────────────────────────────────────────────────────────┘

📋 优先级建议：
1. 🔴 优先处理：理由3（公开不充分）
2. ⚠️ 其次处理：理由2（创造性）
3. 📌 最后处理：理由1（新颖性）

🤝 请您确认：
- 我的理解是否准确？
- 是否遗漏了重要问题？
- 优先级排序是否合理？

【您可以选择】：
A. 确认无误，继续下一步
B. 调整理解（请说明）
C. 查看详细分析
D. 暂停，我稍后继续
```

---

### ⚡ Turn 4: 并行查询（补充信息）

```python
# 并行查询对比文件详情
async def query_cited_documents_parallel(citations):
    """并行查询对比文件详情"""

    # 为每个对比文件并行查询
    query_tasks = [
        query_document_details(citation) for citation in citations
    ]

    results = await parallel(query_tasks)
    return results

async def query_document_details(citation):
    """查询单个对比文件详情（内部也是并行调用）"""

    # 并行获取多个来源的信息
    results = await parallel([
        # 查询1: PostgreSQL数据库
        query_postgresql("""
            SELECT * FROM patents
            WHERE publication_number = %s
        """, (citation["pub_number"],)),

        # 查询2: Qdrant向量相似度
        qdrant_search(
            collection_name="patent_full_text",
            query_vector=embed(citation["title"]),
            top_k=1
        ),

        # 查询3: 全文内容（如果需要）
        get_full_text(citation["pub_number"])
    ])

    db_record, vector_match, full_text = results
    return {
        "citation": citation,
        "db_record": db_record,
        "vector_match": vector_match,
        "full_text": full_text
    }
```

**输出格式**:
```markdown
【第四步: 对比文件并行查询完成】

✅ 已完成并行查询：
- D1: CN1234567A - 完整信息已获取
- D2: CN2345678A - 完整信息已获取
- D3: CN3456789A - 完整信息已获取
- D4: CN4567890A - 完整信息已获取
- D5: 论文XXX - 完整信息已获取

对比文件分析表：

| 对比文件编号 | 文件类型 | 公开日 | 公开号 | 相关权利要求 | 相似度评估 |
|-------------|---------|-------|-------|-------------|-----------|
| D1 | 中国专利 | YYYY-MM-DD | CNXXXXX | 权1-3 | 85% |
| D2 | 中国专利 | YYYY-MM-DD | CNXXXXX | 权4 | 60% |
| D3 | 中国专利 | YYYY-MM-DD | CNXXXXX | - | 40% |
```

---

### ⚡ Turn 5: 顺序分解（需要前序结果）

```python
# 问题分解（顺序执行，需要前序结果）
async def decompose_problems_sequential(analysis_results):
    """问题分解（需要前序分析结果）"""

    # 顺序执行（因为后面的分解可能需要前面的结果）
    decomposed_problems = []

    for rejection in analysis_results["rejections"]:
        # 基于前面的分析结果，进一步分解
        sub_problems = await decompose_rejection(rejection, analysis_results)
        decomposed_problems.append({
            "rejection": rejection,
            "sub_problems": sub_problems
        })

    return decomposed_problems

async def decompose_rejection(rejection, context):
    """分解单个驳回理由（内部可以并行）"""

    # 根据驳回理由类型，生成不同的子问题
    if rejection["type"] == "novelty":
        return await decompose_novelity_rejection(rejection, context)
    elif rejection["type"] == "inventive":
        return await decompose_inventive_rejection(rejection, context)
    # ... 其他类型
```

**🤝 人机交互点3：确认问题分解**

```
【云熙】用户，我已完成问题分解。

📊 问题结构总览：

共有3个驳回理由，分解为12个子问题：
- 理由1: 4个子问题
- 理由2: 4个子问题
- 理由3: 4个子问题

🤝 建议分析顺序：
由于理由3（公开不充分）是根本性问题，建议：
1. 先分析理由3：如果公开不充分成立，无法修改
2. 再分析理由1、2：在公开充分的基础上判断

【您希望】：
A. 按我的建议顺序分析
B. 按审查意见的顺序分析
C. 自定义分析顺序
D. 查看某个理由的详细分解
```

---

## 输出成果

### 成果1：驳回理由摘要表

```markdown
## 审查意见摘要

| 项目 | 内容 |
|-----|------|
| 申请号 | CNXXXXXXXXX.X |
| 审查轮次 | 第X次审查意见 |
| 驳回理由总数 | X个 |
| 最严重问题 | {{理由类型}} |
| 建议答复期限 | YYYY-MM-DD前 |
| 估计答复难度 | 高/中/低 |

## 并行处理统计

- Turn 1（并行读取）: 4个文档同时读取
- Turn 2（并行提取）: 5个信息同时提取
- Turn 3（并行分析）: 3个驳回理由同时分析
- Turn 4（并行查询）: 5个对比文件同时查询
- Turn 5（顺序分解）: 12个子问题顺序分解

**总耗时**: 约30秒（相比顺序处理的120秒，提速75%）
```

### 成果2：详细问题清单

```markdown
## 详细问题清单

### 问题1：{{问题标题}}
- **驳回理由类型**: {{类型}}
- **涉及权利要求**: {{权利要求编号}}
- **法条依据**: {{具体法条}}
- **审查员观点**: {{摘要}}
- **对比文件**: {{引用文件列表}}
- **严重程度**: [高/中/低]
- **建议策略**: {{初步建议}}
- **相似案例**: {{找到X件相似案例}}

### 问题2：{{问题标题}}
...
```

---

## 质量检查清单

{{ skeleton('quality_checklist') }}

**本任务额外检查项**：
- {{}} **并行调用正确**: Turn 1-4使用了并行调用
- {{}} **是否准确理解了审查员的每个观点？**
- {{}} **是否遗漏了任何驳回理由？**
- {{}} **是否准确识别了所有对比文件？**
- {{}} **问题分解是否合理？**
- {{}} **优先级排序是否恰当？**
- {{}} **是否为用户提供了清晰的选择？**
- {{}} **是否检索了相似案例？**

---

## 并行调用性能对比

| 操作模式 | 耗时 | 提速 |
|---------|------|------|
| 顺序处理（旧版） | ~120秒 | 基准 |
| 并行处理（新版） | ~30秒 | **75%** |

**并行化策略**:
- Turn 1: 4个文档并行读取
- Turn 2: 5个信息并行提取
- Turn 3: 3个驳回理由并行分析
- Turn 4: 5个对比文件并行查询
- Turn 5: 顺序分解（需要前序结果）

---

## 下一步

完成本任务后，进入：
**TASK_2_2: 驳回理由深度分析**

对于每个驳回理由进行深入的法律分析和案例检索。

---

**这就是业务层(L4)任务2.1（v2.0 - with parallel calls + Playbook 编排模式）。**
