---
name: legal-world-model
description: 三层法律知识智能问答系统，支持法条查询、案例检索和语义推理
version: "1.0"
---

# Legal World Model Skill (法律世界模型技能)

基于三层架构的法律知识智能问答系统，从法律知识库中提供智能检索和推理服务。

## 🏛️ 技能概述

本技能提供对法律世界模型的访问，包括：
- **三层架构**: 基础法律层(民法典、民诉法)、专利专业层(专利法、审查指南)、司法案例层(判决文书)
- **数据规模**: 32,720文档 + 453,980向量点 + 295,757知识图谱节点
- **检索能力**: 语义向量检索、知识图谱推理、混合查询

## 📊 数据库结构

| 数据库 | 数据量 | 用途 |
|--------|--------|------|
| PostgreSQL | 32,720 文档 | 法律条文、案例文书存储 |
| Qdrant | 453,980 向量 | 语义向量检索 |
| Neo4j | 295,757 节点 | 知识图谱关系推理 |
| Redis | 缓存 | 高速缓存 |

## 🎯 支持的查询类型

### 1. 法律条文查询

查询法律法规、司法解释等内容。

**性能**: < 100ms
**适用场景**: 查询具体法条、法律定义

```python
from skills.legal_world_model.scripts.legal_qa_client import LegalWorldModelClient

client = LegalWorldModelClient()
result = client.ask(
    question="专利法中关于新颖性的规定是什么？",
    query_type="statute_query"
)
```

### 2. 案例检索

检索相关的司法判决、复审决定等案例。

**性能**: < 200ms
**适用场景**: 查找类似案例、判例参考

```python
result = client.ask(
    question="查找关于等同侵权的相关案例",
    query_type="case_query",
    options={
        "max_results": 10,
        "target_layers": ["layer3"]  # 司法案例层
    }
)
```

### 3. 语义问答

基于三层架构进行综合分析和推理。

**性能**: 1-3秒
**适用场景**: 复杂法律问题分析

```python
result = client.ask(
    question="专利复审程序中，如何修改权利要求书？",
    query_type="semantic_qa",
    options={
        "enable_reasoning": True,
        "max_evidence": 10
    }
)
```

## 🚀 使用方法

### 方法1: Python客户端

```python
from skills.legal_world_model.scripts.legal_qa_client import LegalWorldModelClient

# 初始化客户端
client = LegalWorldModelClient(
    api_url="http://localhost:8000",
    timeout=30
)

# 发起查询
result = client.ask(
    question="专利侵权判定中如何确定等同特征？",
    query_type="semantic_qa"
)

# 输出结果
print(f"答案: {result['answer']}")
print(f"置信度: {result['confidence']}")
print(f"参考来源: {len(result['references'])} 条")
```

### 方法2: REST API调用

```bash
# 智能问答
curl -X POST http://localhost:8000/api/v1/qa/ask \
  -H "Content-Type: application/json" \
  -d '{
    "question": "专利法中关于新颖性的规定",
    "query_type": "statute_query"
  }'
```

### 方法3: 命令行工具

```bash
# 使用交互式问答工具
python -m skills.legal_world_model.scripts.legal_qa_cli

# 或直接查询
python -m skills.legal_world_model.scripts.legal_qa_cli \
  --question "什么是专利侵权？" \
  --query-type semantic_qa
```

## 📈 性能基准

| 查询类型 | 响应时间 | 准确率 | 适用场景 |
|---------|---------|--------|----------|
| 法条查询 | < 100ms | 95%+ | 精确检索 |
| 案例检索 | < 200ms | 90%+ | 判例参考 |
| 语义问答 | 1-3秒 | 85%+ | 复杂分析 |
| 混合查询 | 2-5秒 | 90%+ | 综合推理 |

## 🔧 高级用法

### 跨层查询

```python
# 同时查询基础法律层和专利专业层
result = client.ask(
    question="专利复审期限是多少？是否有法律规定？",
    options={
        "target_layers": ["layer1", "layer2"],
        "enable_reasoning": True
    }
)
```

### 证据链展示

```python
# 获取推理过程和证据链
result = client.ask(
    question="分析这个技术方案是否具备创造性",
    options={
        "show_reasoning_chain": True,
        "max_evidence": 15
    }
)

# 遍历推理步骤
for step in result.get('reasoning_chain', []):
    print(f"步骤{step['step_number']}: {step['description']}")
    print(f"  结论: {step['conclusion']}")
    print(f"  置信度: {step['confidence']}")
```

## 📋 API端点

| 端点 | 方法 | 功能 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/api/v1/qa/ask` | POST | 智能问答 |
| `/api/v1/qa/stats` | GET | 统计信息 |

## ⚠️ 注意事项

1. **数据时效**: 法律数据库需定期更新以保持最新
2. **推理时间**: 复杂推理问题可能需要3-5秒
3. **置信度**: 建议关注返回结果的confidence字段
4. **缓存策略**: 常见问题会被缓存，提升响应速度

## 🔗 相关技能

- Patent Retrieval - 专利检索技能
- Prompt System - 动态提示词系统
- Scenario Planner - 场景规划器
