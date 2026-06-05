# 翰林院（Hanlin Academy）

> 多模型集体审议 Skill —— 将问题交给多位国产顶尖 LLM「学士」，匿名互评后由「大学士」综合裁决。

## 命名由来

翰林院是中国古代汇聚天下最优秀学者的最高学术机构。各路才子汇聚一堂、切磋论道、取其精华——正对应多模型审议、匿名互评、最终综合的核心机制。

## 适用场合

| 场景 | 说明 |
|------|------|
| **复杂决策** | 多方案比选、风险评估、战略规划 |
| **知识验证** | 事实核查、多角度交叉验证 |
| **创意发散** | 头脑风暴、多视角碰撞 |
| **代码审查** | 多模型交叉 review 质量、安全性、性能 |
| **文档评审** | 多维度评估文本准确性、完整性、可读性 |
| **学术/法律分析** | 多角度论证、论据交叉检验 |
| **通用问答** | 任何需要高质量、多角度验证的问题 |

**不适用**：简单闲聊、单字翻译等无需多角度验证的轻量任务。

## 三阶段审议流程

```
Stage 1: 各抒己见 → Stage 2: 匿名互评 → Stage 3: 综合定论
```

1. **各抒己见**：用户 query 并行发给所有学士，收集独立回答
2. **匿名互评**：回答匿名化为 A/B/C/D，学士互相评价排名
3. **综合定论**：大学士综合所有观点和排名，输出最终答案

## 默认模型配置

| 角色 | 模型 | 厂商 | 特点 |
|------|------|------|------|
| 🔴 学士 | DeepSeek V4 Flash | 深度求索 | 速度快、性价比高 |
| 🔵 学士 | DeepSeek V4 Pro | 深度求索 | 推理深度强 |
| 🟢 学士 | GLM-5.1 | 智谱 | 中文理解出色 |
| 🟡 学士 | Qwen 3.6-27B | 阿里 | 通用能力均衡 |
| 👑 大学士 | DeepSeek V4 Pro | 深度求索 | 综合裁决 |

所有模型通过**硅基流动（SiliconFlow）** API 调用。

## 环境配置

```bash
# 必需：硅基流动 API Key
export SILICONFLOW_API_KEY="sk-..."
```

## 使用方法

### 方式一：YunXi REPL

```
Skill({"skill": "hanlin-academy"})
```

### 方式二：HTTP API

启动服务：
```bash
cd /path/to/YunXi
python -m assets.skills.technique.hanlin-academy.backend.server
```

发起审议：
```bash
curl -X POST http://localhost:8010/api/hanlin/deliberate 
  -H "Content-Type: application/json" 
  -d '{"query": "你的问题"}'
```

健康检查：
```bash
curl http://localhost:8010/api/hanlin/health
```

## API 响应格式

```json
{
  "stage1": [
    {"model": "deepseek-ai/DeepSeek-V4-Flash", "response": "..."},
    {"model": "deepseek-ai/DeepSeek-V4-Pro", "response": "..."},
    {"model": "Pro/zai-org/GLM-5.1", "response": "..."},
    {"model": "Qwen/Qwen3.6-27B", "response": "..."}
  ],
  "stage2": [
    {"model": "...", "ranking": "...", "parsed_ranking": ["Response C", "Response A", ...]},
    ...
  ],
  "stage3": {
    "model": "deepseek-ai/DeepSeek-V4-Pro",
    "response": "最终综合答案..."
  },
  "metadata": {
    "label_to_model": {"Response A": "...", "Response B": "...", ...},
    "aggregate_rankings": [
      {"model": "...", "average_rank": 1.3, "rankings_count": 3},
      ...
    ]
  }
}
```

## 自定义配置

可通过 POST body 的 `config` 字段自定义：

```json
{
  "query": "你的问题",
  "config": {
    "scholars": ["deepseek-ai/DeepSeek-V4-Flash", "Pro/zai-org/GLM-5.1"],
    "chairman": "deepseek-ai/DeepSeek-V4-Pro"
  }
}
```

## 技术栈

- **后端**: FastAPI + httpx (async) + 硅基流动 API
- **API 格式**: OpenAI-compatible (chat/completions)
- **端口**: 8010
- **无状态**: 每次审议独立，不维护对话历史
