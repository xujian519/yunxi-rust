---
name: patent-retrieval
description: PostgreSQL 专利数据库检索技能，支持关键词、IPC分类、申请人等多种检索方式
version: "1.0"
---

# Patent Retrieval Skill (专利检索技能)

从本地7500万专利数据库中检索专利信息。

## 🎯 技能概述

本技能提供对本地PostgreSQL专利数据库(patent_db)的快速检索访问，包含75,217,242条专利记录(228 GB)。支持多种检索方式：关键词搜索、IPC分类检索、申请人检索、全文检索等。

## 🔍 支持的检索类型

### 1. 关键词搜索 (Keyword Search)

按专利名称、摘要等字段进行模糊匹配检索。

**性能**: < 10ms
**适用场景**: 快速查找特定技术领域的专利

```sql
-- 查询包含"人工智能"的专利
SELECT patent_name, applicant, ipc_main_class
FROM patents
WHERE patent_name ILIKE '%人工智能%'
LIMIT 10;
```

### 2. IPC分类检索 (IPC Classification Search)

按国际专利分类号(IPC)检索专利。

**性能**: < 10ms
**适用场景**: 按技术领域批量检索

```sql
-- 检索G06F(电数字数据处理)分类的专利
SELECT patent_name, ipc_main_class, applicant
FROM patents
WHERE ipc_main_class LIKE 'G06F%'
LIMIT 10;
```

### 3. 申请人检索 (Applicant Search)

按申请人或专利权人检索专利。

**性能**: < 200ms
**适用场景**: 查询特定公司或个人的专利

```sql
-- 检索腾讯公司的专利
SELECT patent_name, application_number, application_date
FROM patents
WHERE applicant ILIKE '%腾讯%'
ORDER BY application_date DESC
LIMIT 10;
```

### 4. 全文检索 (Full-Text Search)

使用预构建的全文索引进行语义检索。

**性能**: < 5秒 (罕见词), 2-3分钟 (常见词)
**适用场景**: 精确语义检索

```sql
-- 全文检索"医疗"
SELECT patent_name, applicant,
       ts_rank(search_vector, query) as relevance
FROM patents, plainto_tsquery('simple', '医疗') query
WHERE search_vector @@ query
ORDER BY relevance DESC
LIMIT 10;
```

## 📊 patents表结构

| 字段名 | 类型 | 说明 |
|--------|------|------|
| id | UUID | 主键 |
| patent_name | text | 专利名称 |
| abstract | text | 摘要 |
| claims_content | text | 权利要求 |
| applicant | text | 申请人 |
| ipc_main_class | text | IPC主分类 |
| application_number | text | 申请号 |
| application_date | date | 申请日期 |
| search_vector | tsvector | 全文索引向量 |

## 🚀 使用方法

### 方法1: 使用psql命令行

```bash
# 连接数据库
psql -h localhost -p 5432 -U postgres -d patent_db

# 执行检索查询
SELECT patent_name, applicant FROM patents
WHERE patent_name ILIKE '%关键词%' LIMIT 10;
```

### 方法2: 使用Python脚本

```python
# 参考 scripts/patent_search_cli.py
from skills.patent_retrieval.scripts.patent_search_cli import PatentSearchCLI

cli = PatentSearchCLI()
results = cli.search_by_keyword("人工智能", limit=10)
```

### 方法3: 使用演示脚本

```bash
# 运行交互式演示
./skills/patent-retrieval/scripts/demo_search.sh
```

## 📈 性能基准

| 检索类型 | 响应时间 | 适用场景 |
|---------|---------|---------|
| 关键词搜索 | < 10ms | 快速查找 |
| IPC分类检索 | < 10ms | 技术分类 |
| 申请人检索 | < 200ms | 主体分析 |
| 中文全文检索 | 2-5秒 | 语义检索 |
| 全文向量检索 | < 5秒 | 精确匹配 |

## 🔧 高级用法

### 组合条件检索

```sql
-- IPC分类 + 关键词组合
SELECT patent_name, applicant
FROM patents
WHERE ipc_main_class LIKE 'A61G%'
  AND patent_name ILIKE '%护理%'
LIMIT 10;
```

### 时间范围检索

```sql
-- 检索近3年的专利
SELECT patent_name, application_date
FROM patents
WHERE application_date >= '2022-01-01'
  AND ipc_main_class LIKE 'G06F%'
ORDER BY application_date DESC
LIMIT 10;
```

### 统计分析

```sql
-- 按年份统计专利数量
SELECT source_year, COUNT(*) as count
FROM patents
WHERE source_year >= 2020
GROUP BY source_year
ORDER BY source_year DESC;
```

## ⚠️ 注意事项

1. **常见词检索**: "人工智能"、"医疗"等常见词的全文检索可能需要2-3分钟
2. **结果限制**: 建议使用LIMIT子句限制返回结果数量
3. **索引覆盖**: 全文检索仅覆盖约37%的记录(2800万条)
4. **向量检索**: 当前未配置，需部署Qdrant后使用

