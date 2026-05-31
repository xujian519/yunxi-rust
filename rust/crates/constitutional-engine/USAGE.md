# 宪法引擎使用示例

## 1. 在专利工具中集成前置检查

```rust
use tools::{ConstitutionalCheckTool, ConstitutionalCheckError};

pub struct PatentDraftingTool {
    checker: ConstitutionalCheckTool,
}

impl PatentDraftingTool {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            checker: ConstitutionalCheckTool::new()?,
        })
    }

    pub fn generate_claims(&self, input: &str) -> Result<String, ConstitutionalCheckError> {
        // 前置检查：保护客体检查
        self.checker.check_input("claim_generator", input, Some("申请前"))?;

        // 生成权利要求...
        let claims = self.do_generate_claims(input)?;

        // 后置检查：撰写质量检查
        let report = self.checker.check_output("claim_generator", &claims, Some("撰写"));

        if report.summary.has_violations {
            self.checker.print_report(&report);
        }

        Ok(claims)
    }
}
```

## 2. 单独使用检查工具

```rust
use tools::ConstitutionalCheckTool;

fn main() {
    let checker = ConstitutionalCheckTool::new().unwrap();

    // 检查保护客体（申请前）
    let content = "一种图像识别装置，包括图像采集模块和处理模块。";
    let report = checker.check_subject_matter(content);

    checker.print_report(&report);

    // 检查撰写质量
    let claims = r#"1. 一种数据处理装置，其特征在于，包括：
   处理模块，用于处理数据；
   存储模块，用于存储数据。

2. 根据权利要求1所述的装置，其特征在于，所述处理模块为CPU。"#;

    let quality_report = checker.check_drafting_quality(claims);
    checker.print_report(&quality_report);
}
```

## 3. 集成到 CLI 工具

```rust
use tools::ConstitutionalCheckTool;
use std::env;

fn main() {
    let checker = ConstitutionalCheckTool::new().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: {} <内容>", args[0]);
        return;
    }

    let content = &args[1];
    let report = checker.check_drafting_quality(content);

    if report.summary.has_violations {
        eprintln!("发现违规！");
        checker.print_report(&report);
        std::process::exit(1);
    } else {
        println!("✅ 所有检查通过");
    }
}
```

## 4. 输出 JSON 格式

```rust
use tools::ConstitutionalCheckTool;

fn main() {
    let checker = ConstitutionalCheckTool::new().unwrap();
    let report = checker.check_subject_matter("一种装置");

    match checker.to_json(&report) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("JSON 序列化失败: {}", e),
    }
}
```

## 5. 自定义配置

```rust
use tools::{ConstitutionalCheckTool, CheckConfig};

fn main() {
    let config = CheckConfig {
        fail_on_block: false,  // 阻塞级违规不抛异常
        default_phase: "审查".to_string(),
        check_all_phases: true,  // 检查所有规则
    };

    let checker = ConstitutionalCheckTool::with_config(config).unwrap();
    let report = checker.check_subject_matter("一种装置");

    checker.print_report(&report);
}
```

## 常见使用场景

### 场景 1：权利要求生成
```rust
// 生成前检查输入
checker.check_input("claim_generator", &tech_content, Some("申请前"))?;

// 生成后检查输出
let claims = generate_claims(&tech_content)?;
checker.check_output("claim_generator", &claims, Some("撰写"))?;
```

### 场景 2：说明书撰写
```rust
checker.check_input("specification_drafter", &tech_content, Some("申请前"))?;

let spec = draft_specification(&tech_content)?;
checker.check_output("specification_drafter", &spec, Some("撰写"))?;
```

### 场景 3：OA 答复
```rust
checker.check_input("oa_response_generator", &oa_notification, Some("答复"))?;

let response = generate_response(&oa_notification)?;
checker.check_output("oa_response_generator", &response, Some("答复"))?;
```

## 输出示例

```
╔═════════════════════════════════════════════════════════╗
║              宪法规则检查报告                              ║
╚═════════════════════════════════════════════════════════╝

📊 检查摘要:
   总检查数: 15
   通过: 13
   失败: 2

⚠️  严重程度统计:
   🔴 Critical: 1
   🟡 Major: 1

❌ 违规详情:

  1. [CON-102] 违法/违反公序良俗排除
     严重程度: Critical
     行动: Block
     法律依据: 专利法第五条：对违反法律、社会公德或妨害公共利益的发明创造，不授予专利权
     详情:
       • 命中禁用词: 赌博
     置信度: 0.90

  2. [CON-101] 发明定义-技术方案三要素
     严重程度: Major
     行动: Warn
     法律依据: 专利法第二条第二款：发明是指对产品、方法或者其改进所提出的新的技术方案
     详情:
       • 缺少要素: technical_problem
     置信度: 0.60

─────────────────────────────────────────────────────────
```

## 下一步

1. 集成到 `patent_drafting` 工具
2. 创建统一检查器 `unified_checker.rs`
3. 编写集成测试
4. 更新文档