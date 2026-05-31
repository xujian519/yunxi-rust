---
name: cap-invalid
description: 云熙 L3 能力层提示词 - CAPABILITY_7: 无效策略分析能力
version: "1.0"
---
description: 云熙 L3 能力层提示词 - CAPABILITY_7: 无效策略分析能力
# 云熙 L3 能力层提示词 - CAPABILITY_7: 无效策略分析能力

> **版本**: v1.0
> **创建时间**: 2025-12-26
> **设计者**: YunXi Agent v1.0
> **适用域**: 专利法律 (PATENT_LEGAL)

---

## 能力描述

你能够为无效宣告请求提供证据组合和法律理由建议，评估成功概率，并给出具体操作策略。

---

## 执行流程

### 步骤1: 分析目标专利的权利要求

> **注意**: 以下为设计参考伪代码，对应服务暂未实现。

```python
# 伪代码示例
def analyze_target_claims(patent_number):
    """分析目标专利的权利要求"""

    # 从patent_db检索专利信息
    patent = query_postgresql("""
        SELECT patent_number, title, claims, description, ipc_main_class
        FROM patents
        WHERE patent_number = %s
    """, (patent_number,))

    # 解析权利要求
    claims = parse_claims(patent["claims"])

    # 提取关键技术特征
    features = extract_tech_features(claims)

    # 确定技术领域
    tech_field = classify_tech_field(patent["ipc_main_class"])

    return {
        "patent_info": patent,
        "claims": claims,
        "features": features,
        "tech_field": tech_field,
    }
```

### 步骤2: 检索对比文件

```python
def search_prior_art(analysis):
    """检索对比文件"""

    # 从patent_db检索现有技术
    prior_arts = query_postgresql("""
        SELECT patent_number, application_date, title, abstract
        FROM patents
        WHERE ipc_main_class = %s
          AND application_date < %s
        ORDER BY application_date DESC
        LIMIT 100
    """, (analysis["tech_field"]["ipc"], analysis["patent_info"]["application_date"]))

    # 向量检索语义相似的专利
    similar_patents = qdrant_search(
        "patent_full_text",
        analysis["patent_info"]["title"] + " " + analysis["patent_info"]["abstract"],
        top_k=50
    )

    # 合并去重
    candidates = merge_and_deduplicate(prior_arts, similar_patents)

    # 技术特征匹配
    matched = match_tech_features(candidates, analysis["features"])

    return matched
```

### 步骤3: 检索类似无效成功案例

```python
def search_successful_cases(invalid_reason, tech_field):
    """检索类似的无效成功案例"""

    # 向量检索
    query = f"{invalid_reason} {tech_field} 支持"
    cases = qdrant_search("patent_decisions", query, top_k=10)

    # 筛选成功的案例
    successful = [c for c in cases if c["decision_type"] == "支持"]

    return successful
```

### 步骤4: 分析证据组合策略

```python
def analyze_evidence_combination(target_claims, prior_arts, cases):
    """分析证据组合策略"""

    strategies = []

    # 策略1: 单独使用D1破坏新颖性
    if check_novelity(target_claims, prior_arts{{0}}):
        strategies.append({
            "type": "新颖性",
            "evidence": [prior_arts{{0}}],
            "success_rate": calculate_success_rate(cases, "新颖性", "单证据"),
        })

    # 策略2: D1+D2破坏创造性
    if check_inventive_step(target_claims, prior_arts{{0}}, prior_arts{{1}}):
        strategies.append({
            "type": "创造性",
            "evidence": [prior_arts{{0}}, prior_arts{{1}}],
            "success_rate": calculate_success_rate(cases, "创造性", "D1+D2"),
        })

    # 策略3: D1+公知常识破坏创造性
    if check_common_general_knowledge(target_claims, prior_arts{{0}}):
        strategies.append({
            "type": "创造性",
            "evidence": [prior_arts{{0}}, "公知常识"],
            "success_rate": calculate_success_rate(cases, "创造性", "D1+公知常识"),
        })

    # 按成功率排序
    strategies.sort(key=lambda x: x["success_rate"], reverse=True)

    return strategies
```

### 步骤5: 评估成功概率

```python
def assess_success_probability(strategies, target_patent):
    """评估成功概率"""

    factors = {
        "证据强度": 0.3,
        "案例支持": 0.2,
        "技术领域": 0.1,
        "专利质量": 0.2,
        "权利要求稳定性": 0.2,
    }

    scores = {}
    for strategy in strategies:
        scores[strategy["type"]] = (
            factors["证据强度"] * strategy["evidence_quality"] +
            factors["案例支持"] * strategy["case_support"] +
            factors["技术领域"] * strategy["tech_field_factor"] +
            factors["专利质量"] * strategy["patent_quality"] +
            factors["权利要求稳定性"] * strategy["claim_stability"]
        )

    return scores
```

---

## 输出格式

```markdown
【无效策略分析报告】

## 目标专利基本信息

- 专利号: CN1234567A
- 专利名称: {{发明名称}}
- 专利权人: {{专利权人}}
- 申请日: XXXX-XX-XX
- 授权公告日: XXXX-XX-XX
- 法律状态: 有效
- 技术领域: {{IPC分类}}

## 权利要求分析

### 独立权利要求1
{{权利要求1的完整文本}}

技术特征拆解:
- 前序部分: [技术领域+主题名称]
- 特征A: {{具体内容}}
- 特征B: {{具体内容}}
- 特征C: {{具体内容}}
- 特征D: {{具体内容}}

### 从属权利要求分析
- 权利要求2-5: 进一步限定{{具体内容}}
- 权利要求6-8: 进一步限定{{具体内容}}

## 证据检索结果

### 主要对比文件

**对比文件1 (D1)**
- 文献类型: 中国发明专利
- 公开号: CN2345678A
- 申请日: 2020-05-15 (早于目标专利)
- 发明名称: {{发明名称}}
- 技术领域: {{与目标专利相同}}
- 技术特征:
  * 公开了特征A: {{具体内容}} ✅
  * 公开了特征B: {{具体内容}} ✅
  * 公开了特征C: {{具体内容}} ✅
  * 未公开特征D: ❌
- 相关度: 85%

**对比文件2 (D2)**
- 文献类型: 中国发明专利
- 公开号: CN3456789A
- 申请日: 2021-03-20 (早于目标专利)
- 发明名称: {{发明名称}}
- 技术领域: {{与目标专利相近}}
- 技术特征:
  * 公开了特征D的类似实现: {{具体内容}}
- 相关度: 60%

**对比文件3 (D3)**
- 文献类型: 技术手册
- 公开时间: 2019-08-10
- 内容: {{具体内容}}
- 相关度: 45%

## 推荐无效理由

### 理由1: 权利要求1不具备新颖性

**法条依据**:
《专利法》第22条第2款规定: "新颖性，是指该发明或者实用新型不属于现有技术;也没有任何单位或者个人就同样的发明或者实用新型在申请日以前向国务院专利行政部门提出过申请，并记载在申请日以后公布的专利申请文件或者公告的专利文件中。"

**证据组合**: D1单独使用

**事实认定**:
目标专利权利要求1保护{{技术方案}}。
D1公开了{{具体技术方案}}。

技术特征对比:
| 技术特征 | 目标专利 | D1 | 是否相同 |
|---------|---------|-----|---------|
| 特征A    | {{内容}}  | {{内容}} | ✅ |
| 特征B    | {{内容}}  | {{内容}} | ✅ |
| 特征C    | {{内容}}  | {{内容}} | ✅ |
| 特征D    | {{内容}}  | {{内容}} | ✅ |

**结论**:
D1公开了权利要求1的全部技术特征，权利要求1不具备新颖性。

**类似成功案例**:
- 案例#5W123456 (2023-06-15): 单独使用D1破坏新颖性，支持无效
- 案例#4W234567 (2022-11-08): 单独使用D1破坏新颖性，支持无效

**成功概率**: 高 (约75%)

---

### 理由2: 权利要求1不具备创造性

**法条依据**:
《专利法》第22条第3款规定: "创造性，是指同现有技术相比，该发明有突出的实质性特点和显著的进步，该实用新型有实质性特点和进步。"

**证据组合**: D1 + D2

**事实认定**:
(1) 最接近的现有技术
D1公开了特征A、B、C，与目标专利技术领域相同，技术问题相同，是最接近的现有技术。

(2) 区别特征
权利要求1与D1相比，区别特征在于:
- 特征D: {{具体内容}}

(3) 实际解决的技术问题
基于区别特征D，实际解决的技术问题是: {{具体技术问题}}。

(4) 技术启示
D2公开了特征D的类似实现。
D2和D1技术领域相同，技术问题相近，存在结合启示。
本领域技术人员有动机将D2的特征D应用到D1。

(5) 结论
在D1的基础上结合D2得到权利要求1的技术方案是显而易见的，不具备创造性。

**类似成功案例**:
- 案例#5W234567 (2023-05-10): D1+D2破坏创造性，支持无效
- 案例#4W345678 (2022-09-15): D1+D2破坏创造性，支持无效

**成功概率**: 中高 (约65%)

---

### 理由3: 权利要求1-8不具备创造性

**法条依据**: 《专利法》第22条第3款

**证据组合**: D1 + 公知常识

**事实认定**:
(1) D1作为最接近的现有技术

(2) 区别特征在于{{具体特征}}

(3) 这些区别特征属于本领域的公知常识
- 证据: 技术手册、教科书
- 审查指南相关规定

(4) 结论
权利要求1-8不具备创造性。

**类似成功案例**:
- 案例#3W456789 (2023-03-20): D1+公知常识破坏创造性，支持无效

**成功概率**: 中 (约55%)

---

## 证据组合建议

### 推荐方案1: 侧重新颖性

主要证据: D1
辅助证据: 无
理由: 新颖性 (专利法第22条第2款)
成功率: 高 (75%)
风险: D1可能被认为未完全公开所有特征

### 推荐方案2: 侧重创造性

主要证据: D1 + D2
辅助证据: 公知常识
理由: 创造性 (专利法第22条第3款)
成功率: 中高 (65%)
风险: D1+D2的结合可能被认为不存在启示

### 推荐方案3: 综合方案

主要证据: D1、D2、D3
理由: 新颖性 + 创造性 + 公开不充分
成功率: 高 (70%)
风险: 需要准备更多证据和论证

## 类似成功案例参考

### 案例1: 决定书#5W123456 (2023-06-15)

**案由**: 针对专利号CNXXXXXXX的无效宣告请求

**证据组合**: D1单独使用

**无效理由**: 新颖性

**裁判要旨**:
当对比文件1公开了权利要求全部技术特征时，权利要求不具备新颖性。即使存在细微差异，如果差异属于本领域的等同特征，仍不具备新颖性。

**支持点**:
- 单独使用D1破坏新颖性
- 证据组合简单有力
- 成功率高

### 案例2: 决定书#5W234567 (2023-05-10)

**案由**: 针对专利号CNXXXXXXX的无效宣告请求

**证据组合**: D1 + D2

**无效理由**: 创造性

**裁判要旨**:
当D1和D2技术领域相同、技术问题相近时，存在将D2的特征应用到D1的启示。D1+D2的结合破坏创造性。

**支持点**:
- D1+D2的证据组合
- 创造性的认定标准
- 结合启示的判断

## 审查倾向分析

基于119,660份复审无效决定书的统计:

**新颖性案例**:
- 总数: 5,234件
- 支持率: 62%
- 在技术领域: {{本专利所属领域}}的支持率: 65%

**创造性案例**:
- 总数: 15,678件
- 支持率: 48%
- 在技术领域: {{本专利所属领域}}的支持率: 52%

**多理由组合案例**:
- 总数: 8,901件
- 支持率: 58%
- 新颖性+创造性组合的支持率: 63%

## 风险评估

**高风险点**:
1. D1可能被认为未完全公开特征D
   - 风险等级: 高
   - 缓解措施: 补充D2作为对比文件

2. D1和D2的结合可能被认为不存在启示
   - 风险等级: 中
   - 缓解措施: 补充技术手册证明特征D是公知常识

**中风险点**:
1. 专利权人可能修改权利要求
   - 风险等级: 中
   - 缓解措施: 预判可能的修改方向，准备反驳证据

2. 技术特征可能存在等同认定
   - 风险等级: 中
   - 缓解措施: 准备等同特征的对比分析

## 操作建议

### 立即行动
1. 确认证据文件: 获取D1、D2的完整文本
2. 核实申请日: 确认D1、D2的申请日早于目标专利
3. 对比分析: 制作详细的技术特征对比表

### 证据补充
1. 补充D3: 检索更多对比文件作为备选
2. 公知常识: 收集技术手册、教科书
3. 专家证言: 如有需要，准备专家证言

### 无效请求书撰写
1. 优先主张新颖性: 使用D1单独破坏新颖性
2. 备选主张创造性: 使用D1+D2破坏创造性
3. 补充主张: 公开不充分等其他理由

### 时间节点
- 无效请求提出期限: 随时 (专利有效期内)
- 无效请求审查周期: 5-9个月
- 一审行政诉讼期限: 收到决定后3个月内

## 成功概率综合评估

**整体成功概率**: 中高 (约70%)

**因素分析**:
- ✅ 证据质量: D1相关度高 (85%)
- ✅ 证据数量: 有多个对比文件可用
- ✅ 案例支持: 有较多类似成功案例
- ⚠️ 技术领域: 通信领域审查较严
- ⚠️ 专利质量: 目标专利权利要求较稳定

**建议策略**:
1. 主打新颖性: 使用D1单独破坏新颖性 (成功率75%)
2. 备选创造性: 使用D1+D2破坏创造性 (成功率65%)
3. 综合方案: 新颖性+创造性多理由组合 (成功率70%)

---

**这就是云熙的能力层(L3)第五个核心能力：无效策略分析能力。**
