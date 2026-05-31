# 宪法引擎

专利法合规规则引擎，支持 20+ 规则类型，基于 YAML 配置，零 LLM 依赖。

## 功能

- **保护客体检查** - 专利法第 2/5/25 条
- **三要素分析** - 技术手段、问题、效果
- **排除客体识别** - 智力活动、诊断方法等
- **说明书分析** - 章节结构、支持性
- **权利要求分析** - 清晰度、依赖关系
- **单一性分析** - 专利法第 31 条
- **修改分析** - 答答期修改限制
- **期限分析** - 宽限期、优先权等

## 规则类型

- `structural_analysis` - 结构分析
- `keyword_blocklist` - 关键词黑名单
- `category_detection` - 类别检测
- `pattern_analysis` - 模式分析
- `specification_analysis` - 说明书维度
- `section_structure` - 章节结构
- `claim_clarity_analysis` - 权利要求清晰度
- `support_analysis` - 支持性分析
- `novelty_analysis` - 新颖性分析
- `inventiveness_analysis` - 创造性分析
- `utility_analysis` - 实用性分析
- `unity_analysis` - 单一性分析
- 等等...

## 使用示例

```rust
use constitutional_engine::{ConstitutionalEngine, RuleLoader};
use std::collections::HashMap;

// 加载规则
let rules = RuleLoader::load_dir(&"assets/constitutional").unwrap();

// 创建引擎
let engine = ConstitutionalEngine::new(rules);

// 执行检查
let results = engine.check_all(
    "claim_generator",
    "一种图像识别装置，包括图像采集模块...",
    Some(&output),
    "撰写"
);

// 处理结果
for result in results {
    if !result.passed {
        println!("违反规则 {}: {}", result.rule_id, result.rule_name);
        println!("  法律依据: {}", result.legal_basis);
        println!("  详情: {:?}", result.details);
    }
}
```

## 规则配置示例

```yaml
subject_matter_invention:
  id: CON-101
  name: 发明定义-技术方案三要素
  severity: critical
  action: block
  legal_basis: 专利法第二条第二款
  phase: 申请前
  check:
    type: structural_analysis
    requires_all:
      - element: technical_means
        patterns: ["装置|设备|系统|单元"]
      - element: technical_problem
        patterns: ["问题|不足|缺陷"]
      - element: technical_effect
        patterns: ["提高|改善|增强"]
```

## 属性说明

- **id** - 规则唯一标识
- **phase** - 生命周期阶段（申请前/撰写/审查/答复/无效/维权）
- **severity** - critical/major/minor
- **action** - block/warn/review/enforce/log
- **legal_basis** - 法律条款原文

## 来源

源自 BCIP 项目 codex-patent-constitutional 引擎。