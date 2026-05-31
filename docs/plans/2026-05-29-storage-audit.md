# 存储系统完整性审计与修复计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标:** 审计 YunXi 项目的 LLM wiki 知识库、知识图谱、向量库及各存储系统，检测完整性和断链，修复发现的问题。

**架构:** 项目采用 SQLite-first 架构，全部存储（向量、知识图谱、文档索引、法律法规、记忆、检查点）均以 SQLite 为统一后端，无外部数据库依赖。唯一网络依赖是本地 BGE-M3 嵌入 HTTP 服务（oMLX 端口 :8009）。

**审计范围:** `assets/` (项目级资产)、`~/.yunxi/` (运行时存储)、`rust/` (代码引用)、BGE-M3 嵌入服务。

---

## 审计结果总览

| 存储系统 | 位置 | 状态 | 问题 |
|---------|------|------|------|
| 语义索引 (.yunpat-semantic-index.sqlite) | `assets/knowledge-base/` | ✅ 完整 | 21179 chunks，1024维嵌入全部完整 |
| 知识图谱 (patent_kg.db) | `assets/knowledge_graph/` | ⚠️ 重复 | 存在两份拷贝，代码只引用旧版 |
| 知识卡片 (card-index.json + 130 .md) | `assets/knowledge-base/cards/` | ✅ 完整 | 索引与文件完全一致 |
| 法律法规 (laws.db / laws-full.db) | `assets/knowledge/data/` | ✅ 完整 | 9121 条记录，26 个分类 |
| 审查指南图谱 (guideline_graph.json) | `assets/knowledge_graph/` | ✅ 完整 | 53 节点 / 51 关系，无悬空引用 |
| 法律实体/关系 JSON | `assets/knowledge_graph/` | ✅ 完整 | 45 实体 / 202 关系，无悬空引用 |
| 向量存储 (vectors.db) | `~/.yunxi/vectors/` | ✅ 可用 | 运行时存储，等待索引填充 |
| 记忆系统 (tiered.sqlite / hebbian.sqlite) | `~/.yunxi/memory/` | ✅ 可用 | 运行时存储，表结构正确 |
| BGE-M3 嵌入模型 | `assets/models/bge-m3/` | ❌ 缺失 | 目录不存在，模型未下载 |
| 嵌入服务 (oMLX :8009) | 本地 HTTP | ⚠️ 需配置 | 服务可达但 settings.local.json 配置为空 |
| 工作流检查点 (checkpoints.db) | `~/.yunxi/workflows/` | ⚠️ 运行时创建 | 首次运行后自动创建，当前不存在属正常 |
| launchd 自动同步 | macOS LaunchAgents | ❌ 未安装 | 知识库同步定时任务未配置 |
| Rust 编译 | 工作区 | ❌ 4 编译错误 | `tools` crate 编译失败 |

---

## 详细问题分析

### 问题 1: Rust 编译错误 (CRITICAL — 阻塞运行)

**文件:** `rust/crates/tools/src/quality_gate/`

- `validators.rs:68` — `#[derive(Serialize)]` 缺少 `use serde::Serialize` 导入
- `validators.rs:69` — `#[serde(...)]` 缺少 serde crate 引用
- `mod.rs:131` 和 `mod.rs:138` — `Self::operator_to_string(threshold.operator)` 类型不匹配（`ThresholdOperator` vs `&ThresholdOperator`）

**影响:** 整个项目无法编译运行。

### 问题 2: patent_kg.db 重复与版本不一致 (HIGH)

存在两份专利知识图谱数据库：

| 路径 | 大小 | 节点数 | 边数 | 修改时间 |
|------|------|--------|------|----------|
| `assets/knowledge_graph/patent_kg.db` | 168 MB | 40,034 | 407,744 | 5月17日 |
| `assets/knowledge-base/patent_kg.db` | 192 MB | 40,309 | 408,185 | 5月26日 |

- **代码引用:** 全部指向 `assets/knowledge_graph/patent_kg.db`（`paths.rs:67`, `patent.rs:522-524`, `sqlite_graph.rs:269` 等 23 处）
- **knowledge-base 中的副本:** 不存在于任何代码路径发现逻辑中
- **差异:** knowledge-base 版本多出 275 个节点和 441 条边，是更新的版本

**影响:** 代码运行时使用的是旧版 knowledge_graph 中的 patent_kg.db，丢失了 275 个节点和 441 条边的数据。且 knowledge-base 副本有活跃 WAL 文件，可能是某个进程正在写入。

### 问题 3: settings.local.json 语义配置为空 (HIGH)

**文件:** `~/.yunxi/settings.local.json` — 当前内容 `{}`

但示例配置 `~/.yunxi/settings.semantic.example.json` 包含正确的嵌入服务配置：

```json
{
  "semantic": {
    "enabled": true,
    "backend": "http",
    "http": {
      "baseUrl": "http://127.0.0.1:8009",
      "embedPath": "/v1/embeddings",
      "apiStyle": "openai",
      "model": "bge-m3-mlx-8bit",
      "apiKey": "YOUR_OMLX_API_KEY",
      "timeoutSecs": 120
    },
    "defaults": {
      "knowledgeSearchMode": "hybrid",
      "semanticCompareAuto": true
    }
  }
}
```

**影响:** 语义搜索功能配置未生效，嵌入服务不会被调用。尽管 oMLX 服务在 8009 端口可达（返回 405/认证错误），但没有配置的情况下代码不会尝试连接。

### 问题 4: BGE-M3 ONNX 模型未下载 (MEDIUM)

**文件:** `assets/models/bge-m3/` — 目录不存在

下载脚本 `scripts/download_models.sh` 存在但未执行。ONNX 后端虽然当前使用 HTTP 远程后端，但如果想切到本地 ONNX 推理则需要模型文件。

### 问题 5: launchd 知识库同步未安装 (MEDIUM)

- `~/Library/LaunchAgents/com.yunxi.*` 不存在
- 安装脚本 `scripts/install-kb-sync-launchd.sh` 已存在但未执行
- 知识库源路径（Obsidian vault + yunpat-agent）均存在且可访问

### 问题 6: WAL 文件残留 (LOW)

- `assets/knowledge-base/patent_kg.db-shm` (32KB) / `patent_kg.db-wal` (0B)
- `assets/knowledge_graph/patent_kg.db-shm` (32KB) / `patent_kg.db-wal` (0B)

表明有进程未正常关闭，WAL 日志未合并到主 DB。

---

## 已通过的检查项 (无需修复)

- ✅ 全部 7 个 SQLite 数据库通过 `PRAGMA integrity_check`
- ✅ 语义索引 21179 chunks，embedding BLOB 无空值（每块 4096 bytes = 1024 f32）
- ✅ 知识卡片 130 张，card-index.json 与文件系统完全一致
- ✅ 法律实体 45 个与关系 202 条，无悬空引用
- ✅ 审查指南图谱 53 节点/51 关系，无悬空引用
- ✅ 法律法规库 9121 条记录，26 个分类类别
- ✅ 语义索引引用的 4410 个源文件路径均实际存在
- ✅ 无断开的符号链接
- ✅ 记忆系统（tiered + hebbian）表结构完整

---

## 修复计划

---

### Task 1: 修复 Rust 编译错误

**文件:**
- Modify: `rust/crates/tools/src/quality_gate/validators.rs:1-2`
- Modify: `rust/crates/tools/src/quality_gate/mod.rs:130-140`

**Step 1: 在 validators.rs 添加缺失的 serde 导入**

修改 `validators.rs` 第 1 行：
```rust
use crate::quality_gate::{QualityGateConfig, QualityGateResult};
use serde::Serialize;
```

**Step 2: 修复 mod.rs 中 operator_to_string 的类型不匹配**

检查 `mod.rs:218` 的 `operator_to_string` 函数签名：
```rust
fn operator_to_string(op: &ThresholdOperator) -> &'static str {
```
调用时 `threshold.operator` 是 `ThresholdOperator`（实现了 Copy），需确认是否自动解引用。如果编译仍报错，添加 `&` 或 deref。

**（实际上）** 更可能的是 `threshold.value` 是 `f64` 但被当作其他类型。需要查看完整错误信息。执行 `cargo check 2>&1` 获取完整错误后精确定位。

**Step 3: 验证编译通过**

Run: `cargo check --workspace`
Expected: 编译成功（仅 warnings 可能保留）

**Step 4: Commit**

```bash
git add rust/crates/tools/src/quality_gate/validators.rs rust/crates/tools/src/quality_gate/mod.rs
git commit -m "fix(tools): 修复 quality_gate 编译错误（缺失 serde 导入 + 类型不匹配）"
```

---

### Task 2: 统一 patent_kg.db 并更新到最新版本

**文件:**
- Copy/Replace: `assets/knowledge_base/patent_kg.db` → `assets/knowledge_graph/patent_kg.db`
- Remove: `assets/knowledge_base/patent_kg.db`
- Clean: `assets/knowledge_base/patent_kg.db-shm`, `patent_kg.db-wal`

**Step 1: 备份旧版并替换**

```bash
# 备份旧版
cp assets/knowledge_graph/patent_kg.db assets/knowledge_graph/patent_kg.db.bak

# 用新版替换
cp assets/knowledge-base/patent_kg.db assets/knowledge_graph/patent_kg.db

# 删除 knowledge-base 中的副本
rm assets/knowledge-base/patent_kg.db assets/knowledge-base/patent_kg.db-shm assets/knowledge-base/patent_kg.db-wal
```

**Step 2: 执行 WAL checkpoint 和 optimize**

```bash
sqlite3 assets/knowledge_graph/patent_kg.db "PRAGMA wal_checkpoint(TRUNCATE);"
sqlite3 assets/knowledge_graph/patent_kg.db "PRAGMA optimize;"
```

**Step 3: 清理旧的 WAL 文件**

```bash
rm -f assets/knowledge_graph/patent_kg.db-shm assets/knowledge_graph/patent_kg.db-wal
```

**Step 4: 验证完整性**

```bash
sqlite3 assets/knowledge_graph/patent_kg.db "PRAGMA integrity_check;"
sqlite3 assets/knowledge_graph/patent_kg.db "SELECT COUNT(*) FROM nodes;"
sqlite3 assets/knowledge_graph/patent_kg.db "SELECT COUNT(*) FROM edges;"
```
Expected: `ok`, 40309 nodes, 408185 edges

**Step 5: Commit**

```bash
git add assets/knowledge_graph/patent_kg.db
git rm assets/knowledge-base/patent_kg.db
git commit -m "fix(storage): 统一 patent_kg.db 至最新版本，移除重复副本"
```

---

### Task 3: 配置语义搜索（settings.local.json）

**文件:**
- Modify: `~/.yunxi/settings.local.json`

**Step 1: 检查 oMLX 服务的 API Key**

检查当前密钥是否有效：
```bash
curl -s http://127.0.0.1:8009/v1/embeddings \
  -H "Authorization: Bearer xj781102@" \
  -H "Content-Type: application/json" \
  -d '{"input":"test","model":"bge-m3-mlx-8bit"}' | head -200
```

**Step 2: 写入语义配置**

将 `settings.local.json` 从 `{}` 更新为包含语义配置的完整内容，使用已验证的 API key（如果上一步返回向量则 key 有效，否则从 `~/.yunxi/secrets.json` 或 `~/.yunxi/settings.json` 查找正确的 key）。

```bash
# 使用示例模板
cat ~/.yunxi/settings.semantic.example.json
```

**Step 3: 验证配置**

重启项目后检查语义搜索功能是否可用。

---

### Task 4: 下载 BGE-M3 ONNX 模型（可选）

**文件:**
- 运行: `scripts/download_models.sh`
- 目标: `assets/models/bge-m3/`

**Step 1: 执行下载脚本**

```bash
bash scripts/download_models.sh
```

**Step 2: 验证模型文件**

```bash
ls -lh assets/models/bge-m3/
```
Expected: 存在 `model.onnx`, `tokenizer.json` 等文件

---

### Task 5: 安装知识库同步定时任务

**文件:**
- 运行: `scripts/install-kb-sync-launchd.sh`

**Step 1: 安装 launchd 任务**

```bash
bash scripts/install-kb-sync-launchd.sh
```

**Step 2: 验证安装**

```bash
launchctl list | grep yunxi
```

---

## 总结

**完整性评估:** 项目的静态资产（知识库、知识图谱、法律法规、语义索引）结构完整，数据一致，无断裂引用。仅有 2 个数据层面的问题（patent_kg.db 重复副本、空配置）和 1 个代码层面的问题（编译错误）。

**可运行性评估:** 当前 **不可运行**，主要阻塞项是 Rust 编译错误（Task 1）。次要阻塞项是语义配置为空（Task 3），但此项目前可能降级为非语义搜索模式运行。
