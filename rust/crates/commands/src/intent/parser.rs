use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use super::types::{Domain, Phase, ScenarioContext, TaskType};

// ============================================================================
// Scenario Identifier
// ============================================================================

/// Rule-based legal scenario identifier (no LLM calls).
#[derive(Debug, Default)]
pub struct ScenarioIdentifier;

impl ScenarioIdentifier {
    /// Create a new scenario identifier.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Identify the scenario from user input.
    #[must_use]
    pub fn identify_scenario(&self, user_input: &str) -> ScenarioContext {
        let (domain, domain_conf) = Self::identify_domain(user_input);
        let (task_type, task_conf) = if domain == Domain::Other {
            (TaskType::Other, 0.0)
        } else {
            Self::identify_task_type(user_input, domain)
        };
        let (phase, _phase_conf) = Self::identify_phase(user_input);

        let confidence = if domain != Domain::Other && task_type != TaskType::Other {
            f64::midpoint(domain_conf, task_conf)
        } else {
            domain_conf
        };

        let extracted_variables = Self::extract_variables(user_input, domain, task_type);

        ScenarioContext {
            domain,
            task_type,
            phase,
            confidence,
            extracted_variables,
            metadata: HashMap::from([("input_length".to_string(), user_input.len().to_string())]),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn identify_domain(text: &str) -> (Domain, f64) {
        let rules: &[(&str, Domain)] = &[
            (
                "专利,发明,实用新型,外观设计,外观,设计,技术方案,权利要求,说明书,实施例,审查意见,创造性,驳回,答复,技术领域,技术问题,交底书",
                Domain::Patent,
            ),
            ("商标,品牌,logo,标识,图形,商号,服务商标", Domain::Trademark),
            ("诉讼,法院,判决,法律,法规,合同,协议", Domain::Legal),
            ("版权,著作权,作品,署名,复制权", Domain::Copyright),
        ];

        let mut best = (Domain::Other, 0usize);
        for (kws, domain) in rules {
            let score = kws.split(',').filter(|kw| text.contains(kw)).count();
            if score > best.1 {
                best = (*domain, score);
            }
        }
        let confidence = (best.1 as f64 * 0.15).min(1.0);
        (best.0, confidence)
    }

    #[allow(clippy::cast_precision_loss)]
    fn identify_task_type(text: &str, domain: Domain) -> (TaskType, f64) {
        let keywords: &[TaskKw] = match domain {
            Domain::Patent => PATENT_TASK_KEYWORDS,
            Domain::Trademark => TRADEMARK_TASK_KEYWORDS,
            Domain::Legal => LEGAL_TASK_KEYWORDS,
            _ => return (TaskType::Other, 0.0),
        };

        let mut best = (TaskType::Other, 0usize);
        for (task, kws) in keywords {
            let score = kws.iter().filter(|kw| text.contains(*kw)).count();
            if score >= best.1 {
                best = (*task, score);
            }
        }
        let confidence = (best.1 as f64 * 0.2).min(1.0);
        (best.0, confidence)
    }

    #[allow(clippy::cast_precision_loss)]
    fn identify_phase(text: &str) -> (Phase, f64) {
        let rules: &[(&[&str], Phase)] = &[
            (&["申请", "提交", "申请文件", "立案"], Phase::Application),
            (
                &["审查", "审查意见", "驳回", "补正", "答复"],
                Phase::Examination,
            ),
            (&["异议", "无效宣告", "复审"], Phase::Opposition),
            (
                &["诉讼", "起诉", "判决", "法院", "法庭", "裁决"],
                Phase::Litigation,
            ),
        ];

        let mut best = (Phase::Other, 0usize);
        for (kws, phase) in rules {
            let score = kws.iter().filter(|kw| text.contains(*kw)).count();
            if score > best.1 {
                best = (*phase, score);
            }
        }
        let confidence = (best.1 as f64 * 0.25).min(1.0);
        (best.0, confidence)
    }

    fn extract_variables(
        text: &str,
        domain: Domain,
        _task_type: TaskType,
    ) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        if domain == Domain::Patent {
            if let Some(c) = TECH_FIELD.captures(text) {
                vars.insert("technology_field".into(), c[1].trim().to_string());
            }
            if let Some(c) = PROBLEM.captures(text) {
                vars.insert("technical_problem".into(), c[1].trim().to_string());
            }
        } else if domain == Domain::Trademark {
            if let Some(c) = TM.captures(text) {
                vars.insert("trademark_name".into(), c[1].trim().to_string());
            }
            if let Some(c) = CAT.captures(text) {
                vars.insert("trademark_category".into(), c[1].to_string());
            }
        }

        if let Some(c) = LEGAL_BASIS.captures(text) {
            vars.insert("legal_basis".into(), c[2].trim().to_string());
        }

        vars
    }
}

// ============================================================================
// Regex patterns for variable extraction
// ============================================================================

static TECH_FIELD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"技术领域[:：]\s*([^。,\n，]+)").expect("static regex pattern is valid")
});
static PROBLEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"技术问题[:：]\s*([^。,\n，]+)").expect("static regex pattern is valid")
});
static TM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"商标[:：]\s*([^。,\n，]+)").expect("static regex pattern is valid")
});
static CAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"第(\d+)类").expect("static regex pattern is valid"));
static LEGAL_BASIS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(法律依据|根据|依据)[:：]\s*([^。,\n，]+)").expect("static regex pattern is valid")
});

// ============================================================================
// Task keyword tables
// ============================================================================

type TaskKw = (TaskType, &'static [&'static str]);

const PATENT_TASK_KEYWORDS: &[TaskKw] = &[
    (
        TaskType::CreativityAnalysis,
        &[
            "创造性",
            "创新性",
            "创新高度",
            "技术贡献",
            "突出的实质性特点",
            "显著的进步",
            "用途发明",
            "预料不到",
            "事后诸葛亮",
            "复审请求",
            "驳回复审",
        ],
    ),
    (
        TaskType::NoveltyAnalysis,
        &["新颖性", "现有技术", "公开", "在先技术", "不属于现有技术"],
    ),
    (
        TaskType::Infringement,
        &["侵权", "落入保护范围", "等同", "相同", "保护范围"],
    ),
    (
        TaskType::Validity,
        &["无效", "无效宣告", "不符合专利法", "不具备", "不授予"],
    ),
    (
        TaskType::Drafting,
        &[
            "撰写",
            "写",
            "起草",
            "生成",
            "申请文件",
            "权利要求",
            "说明书",
            "摘要",
        ],
    ),
    (
        TaskType::Search,
        &["检索", "查新", "现有技术检索", "对比文件"],
    ),
];

const TRADEMARK_TASK_KEYWORDS: &[TaskKw] = &[
    (
        TaskType::Similarity,
        &["相似", "近似", "混淆", "容易误认", "视觉近似", "读音近似"],
    ),
    (
        TaskType::Infringement,
        &["侵权", "擅自使用", "相同或相似", "容易导致混淆"],
    ),
    (
        TaskType::Validity,
        &["无效", "撤销", "显著性", "缺乏显著性"],
    ),
    (TaskType::Drafting, &["申请", "注册", "商标申请", "图样"]),
];

const LEGAL_TASK_KEYWORDS: &[TaskKw] = &[
    (
        TaskType::Infringement,
        &["侵权", "损害赔偿", "停止侵害", "法律责任"],
    ),
    (TaskType::Validity, &["效力", "无效", "可撤销"]),
    (TaskType::Drafting, &["合同", "起草", "法律文书", "协议"]),
];

// ============================================================================
// Convenience function
// ============================================================================

/// Convenience function to identify a scenario from raw user input.
#[must_use]
pub fn identify_scenario_from_input(user_input: &str) -> ScenarioContext {
    ScenarioIdentifier::new().identify_scenario(user_input)
}
