use constitutional_engine::{ConstitutionalEngine, RuleAction, RuleLoader, RuleSeverity};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub enum ConstitutionalCheckError {
    LoadError(String),
    BlockViolation(Vec<String>),
    CheckError(String),
}

impl std::fmt::Display for ConstitutionalCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstitutionalCheckError::LoadError(s) => write!(f, "规则加载失败: {}", s),
            ConstitutionalCheckError::BlockViolation(v) => {
                write!(f, "违阻违规，无法继续: {:?}", v)
            }
            ConstitutionalCheckError::CheckError(s) => write!(f, "检查执行失败: {}", s),
        }
    }
}

impl std::error::Error for ConstitutionalCheckError {}

/// 宪法规则检查工具
///
/// 在专利工具调用前后检查是否违反专利法规则
pub struct ConstitutionalCheckTool {
    engine: ConstitutionalEngine,
    config: CheckConfig,
}

#[derive(Debug, Clone)]
pub struct CheckConfig {
    /// 是否在检测到 block 级违规时抛出异常
    pub fail_on_block: bool,
    /// 默认检查阶段
    pub default_phase: String,
    /// 是否检查所有规则（忽略 phase 过滤）
    pub check_all_phases: bool,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            fail_on_block: true,
            default_phase: "撰写".to_string(),
            check_all_phases: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckReport {
    pub summary: CheckSummary,
    pub results: Vec<CheckResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub has_violations: bool,
    pub critical_count: usize,
    pub major_count: usize,
    pub minor_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: String,
    pub action: String,
    pub legal_basis: String,
    pub passed: bool,
    pub details: Vec<String>,
    pub confidence: f64,
}

impl ConstitutionalCheckTool {
    /// 创建新的宪法检查工具
    ///
    /// 自动从 assets/constitutional 目录加载规则
    pub fn new() -> Result<Self, ConstitutionalCheckError> {
        Self::with_config(CheckConfig::default())
    }

    /// 使用指定配置创建检查工具
    pub fn with_config(config: CheckConfig) -> Result<Self, ConstitutionalCheckError> {
        let rules = Self::load_rules().map_err(ConstitutionalCheckError::LoadError)?;
        if rules.is_empty() {
            return Err(ConstitutionalCheckError::LoadError(
                "未找到任何规则文件".to_string(),
            ));
        }

        let engine = ConstitutionalEngine::new(rules);

        Ok(Self { engine, config })
    }

    /// 从指定路径加载规则
    pub fn from_path(paths: &[PathBuf]) -> Result<Self, ConstitutionalCheckError> {
        let rules = RuleLoader::load_rules_from(paths)
            .map_err(|e| ConstitutionalCheckError::LoadError(e.to_string()))?;

        if rules.is_empty() {
            return Err(ConstitutionalCheckError::LoadError(
                "未找到任何规则文件".to_string(),
            ));
        }

        let engine = ConstitutionalEngine::new(rules);

        Ok(Self {
            engine,
            config: CheckConfig::default(),
        })
    }

    /// 检查工具输入（前置检查）
    ///
    /// 用于在工具执行前检查输入是否符合专利法要求
    pub fn check_input(
        &self,
        tool_name: &str,
        input_text: &str,
        phase: Option<&str>,
    ) -> Result<CheckReport, ConstitutionalCheckError> {
        let phase = phase.unwrap_or(&self.config.default_phase);
        let results = self.engine.check_all(tool_name, input_text, None, phase);

        let blocking_violations: Vec<String> = results
            .iter()
            .filter(|r| !r.passed && matches!(r.action, RuleAction::Block))
            .map(|r| format!("{}: {}", r.rule_id, r.rule_name))
            .collect();

        if !blocking_violations.is_empty() && self.config.fail_on_block {
            return Err(ConstitutionalCheckError::BlockViolation(
                blocking_violations,
            ));
        }

        Ok(self.format_report(results))
    }

    /// 检查工具输出（后置检查）
    ///
    /// 用于在工具执行后检查输出是否符合专利法要求
    pub fn check_output(
        &self,
        tool_name: &str,
        output_text: &str,
        phase: Option<&str>,
    ) -> CheckReport {
        let phase = phase.unwrap_or(&self.config.default_phase);
        let results = self.engine.check_all(tool_name, output_text, None, phase);

        self.format_report(results)
    }

    /// 检查输入和输出（完整检查）
    pub fn check_full(
        &self,
        tool_name: &str,
        input_text: &str,
        output_text: &str,
        phase: Option<&str>,
    ) -> Result<CheckReport, ConstitutionalCheckError> {
        self.check_input(tool_name, input_text, phase)?;
        let output_report = self.check_output(tool_name, output_text, phase);
        Ok(output_report)
    }

    /// 按指定阶段检查
    pub fn check_by_phase(&self, text: &str, phase: &str) -> CheckReport {
        let results = self.engine.check_all("manual_check", text, None, phase);
        self.format_report(results)
    }

    /// 检查文本是否包含违阻内容（保护客体）
    pub fn check_subject_matter(&self, text: &str) -> CheckReport {
        self.check_by_phase(text, "申请前")
    }

    /// 检查专利撰写质量
    pub fn check_drafting_quality(&self, text: &str) -> CheckReport {
        self.check_by_phase(text, "撰写")
    }

    /// 检查审查意见答复
    pub fn check_oa_response(&self, text: &str) -> CheckReport {
        self.check_by_phase(text, "答复")
    }

    /// 加载规则文件
    fn load_rules() -> Result<HashMap<String, constitutional_engine::ConstitutionalRules>, String> {
        let mut paths = vec![];

        // 查找规则目录
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let candidates = vec![
                PathBuf::from(&manifest_dir)
                    .join("../../crates/constitutional-engine/assets/constitutional"),
                PathBuf::from(&manifest_dir).join("../constitutional-engine/assets/constitutional"),
                PathBuf::from("assets/constitutional"),
            ];

            for candidate in candidates {
                if candidate.is_dir() {
                    paths.push(candidate);
                    break;
                }
            }
        }

        // 备用路径：从工作目录查找
        if paths.is_empty() {
            let work_paths = vec![
                PathBuf::from("rust/crates/constitutional-engine/assets/constitutional"),
                PathBuf::from("crates/constitutional-engine/assets/constitutional"),
            ];

            for candidate in work_paths {
                if candidate.is_dir() {
                    paths.push(candidate);
                    break;
                }
            }
        }

        if paths.is_empty() {
            return Err("找不到宪法规则目录 (assets/constitutional)".to_string());
        }

        RuleLoader::load_rules_from(&paths).map_err(|e| e.to_string())
    }

    /// 格式化检查报告
    fn format_report(&self, results: Vec<constitutional_engine::RuleCheckResult>) -> CheckReport {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;

        let critical_count = results
            .iter()
            .filter(|r| matches!(r.severity, RuleSeverity::Critical))
            .count();
        let major_count = results
            .iter()
            .filter(|r| matches!(r.severity, RuleSeverity::Major))
            .count();
        let minor_count = results
            .iter()
            .filter(|r| matches!(r.severity, RuleSeverity::Minor))
            .count();

        let check_results = results
            .into_iter()
            .map(|r| CheckResult {
                rule_id: r.rule_id,
                rule_name: r.rule_name,
                severity: format!("{:?}", r.severity),
                action: format!("{:?}", r.action),
                legal_basis: r.legal_basis,
                passed: r.passed,
                details: r.details,
                confidence: r.confidence,
            })
            .collect();

        CheckReport {
            summary: CheckSummary {
                total_checks: passed + failed,
                passed,
                failed,
                has_violations: failed > 0,
                critical_count,
                major_count,
                minor_count,
            },
            results: check_results,
        }
    }

    /// 获取 JSON 格式报告
    pub fn to_json(&self, report: &CheckReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// 打印报告到控制台
    pub fn print_report(&self, report: &CheckReport) {
        println!("\n╔═════════════════════════════════════════════════════════╗");
        println!("║              宪法规则检查报告                              ║");
        println!("╚═════════════════════════════════════════════════════════╝");

        println!("\n📊 检查摘要:");
        println!("   总检查数: {}", report.summary.total_checks);
        println!("   通过: {}", report.summary.passed);
        println!("   失败: {}", report.summary.failed);

        if report.summary.failed > 0 {
            println!("\n⚠️  严重程度统计:");
            if report.summary.critical_count > 0 {
                println!("   🔴 Critical: {}", report.summary.critical_count);
            }
            if report.summary.major_count > 0 {
                println!("   🟡 Major: {}", report.summary.major_count);
            }
            if report.summary.minor_count > 0 {
                println!("   🟢 Minor: {}", report.summary.minor_count);
            }
        }

        let violations: Vec<_> = report.results.iter().filter(|r| !r.passed).collect();

        if !violations.is_empty() {
            println!("\n❌ 违规详情:");
            for (i, violation) in violations.iter().enumerate() {
                println!(
                    "\n  {}. [{}] {}",
                    i + 1,
                    violation.rule_id,
                    violation.rule_name
                );
                println!("     严重程度: {}", violation.severity);
                println!("     行动: {}", violation.action);
                if !violation.legal_basis.is_empty() {
                    println!("     法律依据: {}", violation.legal_basis);
                }
                println!("     详情:");
                for detail in &violation.details {
                    println!("       • {}", detail);
                }
                println!("     置信度: {:.2}", violation.confidence);
            }
        } else {
            println!("\n✅ 未发现违规，所有检查通过！");
        }

        println!("\n{}", "─".repeat(59));
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tool() {
        match ConstitutionalCheckTool::new() {
            Ok(_) => println!("✅ 工具创建成功"),
            Err(ref e) => println!("❌ 工具创建失败: {}", e),
        }
        let tool = ConstitutionalCheckTool::new();
        assert!(tool.is_ok(), "工具创建失败");
    }

    #[test]
    fn test_check_subject_matter() {
        let tool = ConstitutionalCheckTool::new().unwrap();

        // 测试违法内容
        let illegal_content = "一种赌博装置，包括控制器和显示器";
        let report = tool.check_subject_matter(illegal_content);

        assert!(report.summary.has_violations, "应检测到违法内容");
        assert!(report.summary.failed > 0, "应有违规记录");
    }

    #[test]
    fn test_check_valid_invention() {
        let tool = ConstitutionalCheckTool::new().unwrap();

        let valid_content =
            "一种图像识别装置，包括图像采集模块和图像处理模块，用于识别图像中的物体。";
        let report = tool.check_subject_matter(valid_content);

        // 可能没有违规，也可能有其他问题
        assert!(report.summary.total_checks > 0, "应执行检查");
    }

    #[test]
    fn test_block_violation() {
        let tool = ConstitutionalCheckTool::with_config(CheckConfig {
            fail_on_block: true,
            ..Default::default()
        })
        .unwrap();

        let illegal_content = "一种赌博装置";
        let result = tool.check_input("test_tool", illegal_content, Some("申请前"));

        assert!(result.is_err(), "应因 block 违规而失败");
        match result {
            Err(ConstitutionalCheckError::BlockViolation(_)) => {}
            _ => panic!("应为 BlockViolation 错误"),
        }
    }

    #[test]
    fn test_check_drafting_quality() {
        let tool = ConstitutionalCheckTool::new().unwrap();

        let claims = r#"1. 一种数据处理装置，其特征在于，包括：
   处理模块，用于处理数据；
   存储模块，用于存储数据。

2. 根据权利要求1所述的装置，其特征在于，所述处理模块为CPU。"#;

        let report = tool.check_drafting_quality(claims);
        assert!(report.summary.total_checks > 0, "应执行检查");
    }

    #[test]
    fn test_to_json() {
        let tool = ConstitutionalCheckTool::new().unwrap();
        let report = tool.check_subject_matter("一种装置");

        let json = tool.to_json(&report);
        assert!(json.is_ok(), "JSON 序列化应成功");
    }

    #[test]
    fn test_print_report() {
        let tool = ConstitutionalCheckTool::new().unwrap();
        let report = tool.check_subject_matter("一种装置");

        tool.print_report(&report);
    }
}
