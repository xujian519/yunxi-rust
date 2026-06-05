use crate::cache::LlmAnalysisCache;
use crate::llm_analyzer::{ConstitutionalLlmAnalyzer, LlmAnalysisResult};
use crate::model::{ConstitutionalRule, ConstitutionalRules, RuleAction, RuleCheck, RuleSeverity};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RuleCheckResult {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: RuleSeverity,
    pub action: RuleAction,
    pub legal_basis: String,
    pub passed: bool,
    pub details: Vec<String>,
    pub confidence: f64,
}

pub struct ConstitutionalEngine {
    rules: HashMap<String, ConstitutionalRules>,
    /// LLM 深度分析器（可选）
    llm_analyzer: Option<Arc<dyn ConstitutionalLlmAnalyzer>>,
    /// LLM 分析结果缓存
    llm_cache: LlmAnalysisCache,
}

impl ConstitutionalEngine {
    pub fn new(rules: HashMap<String, ConstitutionalRules>) -> Self {
        Self {
            rules,
            llm_analyzer: None,
            llm_cache: LlmAnalysisCache::new(),
        }
    }

    /// Builder: 注入 LLM 分析器。
    pub fn with_llm_analyzer(mut self, analyzer: Arc<dyn ConstitutionalLlmAnalyzer>) -> Self {
        self.llm_analyzer = Some(analyzer);
        self
    }

    pub fn check_all(
        &self,
        tool_name: &str,
        input_text: &str,
        output_text: Option<&str>,
        phase: &str,
    ) -> Vec<RuleCheckResult> {
        let mut results = Vec::new();
        for ruleset in self.rules.values() {
            for rule in ruleset.rules.values() {
                if !rule.phase.is_empty() && rule.phase != phase {
                    continue;
                }
                let result = self.evaluate_rule(rule, tool_name, input_text, output_text);
                results.push(result);
            }
        }
        results
    }

    /// 带缓存的 check_all，LLM 分析结果会被缓存。
    pub fn check_all_cached(
        &mut self,
        tool_name: &str,
        input_text: &str,
        output_text: Option<&str>,
        phase: &str,
    ) -> Vec<RuleCheckResult> {
        // 先收集需要评估的规则（避免与 self.llm_cache 的借用冲突）
        let rules_to_check: Vec<ConstitutionalRule> = self
            .rules
            .values()
            .flat_map(|ruleset| ruleset.rules.values().cloned())
            .filter(|rule| rule.phase.is_empty() || rule.phase == phase)
            .collect();

        let mut results = Vec::new();
        for rule in &rules_to_check {
            let result = self.evaluate_rule_cached(rule, tool_name, input_text, output_text);
            results.push(result);
        }

        // 定期清理过期缓存
        let evicted = self.llm_cache.evict_expired();
        if evicted > 0 {
            eprintln!("[constitutional-engine] 清理过期 LLM 缓存: {} 条", evicted);
        }

        results
    }

    fn evaluate_rule(
        &self,
        rule: &ConstitutionalRule,
        _tool_name: &str,
        input_text: &str,
        _output_text: Option<&str>,
    ) -> RuleCheckResult {
        let severity = RuleSeverity::from_str(&rule.severity).unwrap_or(RuleSeverity::Minor);
        let action = RuleAction::from_str(&rule.action).unwrap_or(RuleAction::Warn);

        let (passed, details, confidence) = match &rule.check {
            None => (true, vec!["规则无自动检查配置，需人工审查".into()], 0.5),
            Some(RuleCheck::StructuralAnalysis {
                requires_all,
                min_confidence,
            }) => {
                let mut missing = Vec::new();
                for elem in requires_all {
                    let has_elem = elem
                        .patterns
                        .iter()
                        .any(|p| input_text.contains(p.trim_matches('"')));
                    if !has_elem {
                        missing.push(elem.element.clone());
                    }
                }
                if missing.is_empty() {
                    (true, vec!["三要素完整".into()], *min_confidence + 0.2)
                } else {
                    (
                        false,
                        missing.iter().map(|m| format!("缺少要素: {}", m)).collect(),
                        *min_confidence,
                    )
                }
            }
            Some(RuleCheck::KeywordBlocklist {
                keywords,
                context_ban,
                absolute_ban,
                ..
            }) => {
                let all_ban: Vec<&String> = keywords
                    .iter()
                    .chain(context_ban.iter())
                    .chain(absolute_ban.iter())
                    .collect();
                let mut found = Vec::new();
                for pattern in &all_ban {
                    if input_text.contains(pattern.trim_matches('"')) {
                        found.push((*pattern).clone());
                    }
                }
                if found.is_empty() {
                    (true, vec!["未命中禁用词".into()], 0.95)
                } else {
                    (
                        false,
                        found.iter().map(|f| format!("命中禁用词: {}", f)).collect(),
                        0.9,
                    )
                }
            }
            Some(RuleCheck::PatternAnalysis {
                pure_software_markers,
                hardware_integration_markers,
                guidance: _,
            }) => {
                let pure_hits: Vec<&String> = pure_software_markers
                    .iter()
                    .filter(|p| input_text.contains(p.trim_matches('"')))
                    .collect();
                let hw_hits: Vec<&String> = hardware_integration_markers
                    .iter()
                    .filter(|p| input_text.contains(p.trim_matches('"')))
                    .collect();
                if !pure_hits.is_empty() && hw_hits.is_empty() {
                    (false, vec!["纯软件方案，需结合硬件分析".into()], 0.7)
                } else {
                    (true, vec!["通过模式分析".into()], 0.85)
                }
            }
            Some(RuleCheck::CategoryDetection {
                categories,
                assessment: _,
            }) => {
                let mut matches = Vec::new();
                for (cat_name, cat_def) in categories {
                    let cat_hits: Vec<&String> = cat_def
                        .patterns
                        .iter()
                        .filter(|p| input_text.contains(p.trim_matches('"')))
                        .collect();
                    if !cat_hits.is_empty() {
                        matches.push(format!("[{}] 命中 {} 个模式", cat_name, cat_hits.len()));
                    }
                }
                if matches.is_empty() {
                    (true, vec!["未命中排除客体类别".into()], 0.9)
                } else {
                    (false, matches, 0.8)
                }
            }
            Some(RuleCheck::SpecificationAnalysis {
                dimensions,
                assessment: _,
            }) => {
                let mut dim_results = Vec::new();
                for dim in dimensions {
                    let all_checks_pass = dim
                        .checks
                        .iter()
                        .all(|c| input_text.contains(c.trim_matches('"')));
                    if !all_checks_pass {
                        dim_results.push(format!("维度 '{}' 未全部满足", dim.dimension));
                    }
                }
                if dim_results.is_empty() {
                    (true, vec!["说明书分析维度全部通过".into()], 0.85)
                } else {
                    (false, dim_results, 0.7)
                }
            }
            Some(RuleCheck::SectionStructure {
                required_sections,
                forbidden_content: _,
            }) => {
                let mut missing_sections = Vec::new();
                for section in required_sections {
                    let found = section
                        .patterns
                        .iter()
                        .any(|p| input_text.contains(p.trim_matches('"')));
                    if !found {
                        missing_sections.push(section.name.clone());
                    }
                }
                if missing_sections.is_empty() {
                    (true, vec!["章节结构完整".into()], 0.9)
                } else {
                    (
                        false,
                        missing_sections
                            .iter()
                            .map(|s| format!("缺少章节: {}", s))
                            .collect(),
                        0.75,
                    )
                }
            }
            _ => {
                // Fallback 分支：尝试使用 LLM 深度分析
                self.evaluate_with_llm(rule, input_text, _output_text)
            }
        };

        // P1-6: confidence < 0.6 的结果调用 LLM 深度分析
        let (final_passed, final_details, final_confidence) = if confidence < 0.6 {
            if let Some(ref analyzer) = self.llm_analyzer {
                match analyzer.analyze(rule, input_text, _output_text, "") {
                    Ok(llm_result) => {
                        eprintln!(
                                "[constitutional-engine] LLM 增强: rule={}, keyword_conf={:.2}, llm_conf={:.2}",
                                rule.id, confidence, llm_result.confidence
                            );
                        (llm_result.passed, llm_result.details, llm_result.confidence)
                    }
                    Err(_) => (passed, details, confidence),
                }
            } else {
                (passed, details, confidence)
            }
        } else {
            (passed, details, confidence)
        };

        RuleCheckResult {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            severity,
            action,
            legal_basis: rule.legal_basis.clone(),
            passed: final_passed,
            details: final_details,
            confidence: final_confidence,
        }
    }

    /// 使用 LLM 深度分析评估规则（_ 分支的增强实现）。
    ///
    /// LLM 不可用时降级为关键词匹配，保持现有行为。
    fn evaluate_with_llm(
        &self,
        rule: &ConstitutionalRule,
        input_text: &str,
        output_text: Option<&str>,
    ) -> (bool, Vec<String>, f64) {
        match self.llm_analyzer {
            Some(ref analyzer) => {
                eprintln!(
                    "[constitutional-engine] LLM 深度分析规则 '{}' (id={})",
                    rule.name, rule.id
                );
                match analyzer.analyze(rule, input_text, output_text, "") {
                    Ok(result) => {
                        eprintln!(
                            "[constitutional-engine] LLM 分析完成: passed={}, confidence={:.2}",
                            result.passed, result.confidence
                        );
                        (result.passed, result.details, result.confidence)
                    }
                    Err(e) => {
                        eprintln!(
                            "[constitutional-engine] LLM 分析失败: {}，降级为关键词匹配",
                            e
                        );
                        // 降级为原有行为
                        (
                            true,
                            vec![format!(
                                "规则 '{}' 需要深度 LLM 辅助检查（LLM 调用失败）",
                                rule.name
                            )],
                            0.5,
                        )
                    }
                }
            }
            None => {
                // LLM 不可用，保持当前关键词匹配逻辑
                (
                    true,
                    vec![format!("规则 '{}' 需要深度 LLM 辅助检查", rule.name)],
                    0.5,
                )
            }
        }
    }

    /// 带缓存的规则评估（用于 `check_all_cached`）。
    fn evaluate_rule_cached(
        &mut self,
        rule: &ConstitutionalRule,
        tool_name: &str,
        input_text: &str,
        output_text: Option<&str>,
    ) -> RuleCheckResult {
        // 对于 _ 分支的规则，尝试使用缓存
        if rule.check.is_some() && !self.is_keyword_matchable(rule) {
            let cache_key = LlmAnalysisCache::make_cache_key(&rule.id, input_text, output_text);

            if let Some(cached_result) = self.llm_cache.get(&cache_key) {
                let severity =
                    RuleSeverity::from_str(&rule.severity).unwrap_or(RuleSeverity::Minor);
                let action = RuleAction::from_str(&rule.action).unwrap_or(RuleAction::Warn);
                eprintln!("[constitutional-engine] LLM 缓存命中: rule={}", rule.id);
                return RuleCheckResult {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    severity,
                    action,
                    legal_basis: rule.legal_basis.clone(),
                    passed: cached_result.passed,
                    details: cached_result.details.clone(),
                    confidence: cached_result.confidence,
                };
            }

            // 缓存未命中，执行评估
            let result = self.evaluate_rule(rule, tool_name, input_text, output_text);

            // 如果使用了 LLM 分析，缓存结果
            if result.confidence > 0.5 || !result.details.is_empty() {
                let llm_result = LlmAnalysisResult {
                    passed: result.passed,
                    confidence: result.confidence,
                    details: result.details.clone(),
                    reasoning: format!("Cached for rule {}", rule.id),
                };
                self.llm_cache.put(cache_key, llm_result);
            }

            return result;
        }

        // 非缓存规则直接评估
        self.evaluate_rule(rule, tool_name, input_text, output_text)
    }

    /// 判断规则是否可以通过关键词匹配处理（不需要 LLM）。
    fn is_keyword_matchable(&self, rule: &ConstitutionalRule) -> bool {
        matches!(
            &rule.check,
            None | Some(RuleCheck::StructuralAnalysis { .. })
                | Some(RuleCheck::KeywordBlocklist { .. })
                | Some(RuleCheck::PatternAnalysis { .. })
                | Some(RuleCheck::CategoryDetection { .. })
                | Some(RuleCheck::SpecificationAnalysis { .. })
                | Some(RuleCheck::SectionStructure { .. })
        )
    }
}
