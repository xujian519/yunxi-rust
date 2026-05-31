use crate::reflection::{ActionIssue, ReflectionConfig, ReflectionResult, Reflector};
use serde_json::Value;

/// Action-Review 循环（MetaGPT 角色评审链）。
///
/// 在每个动作执行后自动插入评审节点。
pub struct ActionReviewReflector {
    config: ReflectionConfig,
}

impl ActionReviewReflector {
    pub fn new(config: ReflectionConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self {
            config: ReflectionConfig::default(),
        }
    }

    /// 评审动作结果。
    fn review_action(&self, result: &Value, action_type: &str) -> ReflectionResult {
        let mut issues = Vec::new();
        let mut success = true;
        let mut score = 100.0;

        // 检查基本状态
        if let Some(error) = result.get("error") {
            if !error.is_null() {
                success = false;
                score = 0.0;
                issues.push(ActionIssue {
                    category: "执行错误".to_string(),
                    severity: "critical".to_string(),
                    description: error.as_str().unwrap_or("未知错误").to_string(),
                    location: None,
                });
            }
        }

        // 检查输出质量（如果适用）
        if let Some(quality_score) = result.get("quality_score") {
            if let Some(q_score) = quality_score.as_f64() {
                if q_score < self.config.score_threshold {
                    score = q_score;
                    issues.push(ActionIssue {
                        category: "质量不足".to_string(),
                        severity: "medium".to_string(),
                        description: format!(
                            "质量评分 {:.1} 低于阈值 {:.1}",
                            q_score, self.config.score_threshold
                        ),
                        location: None,
                    });
                } else {
                    score = q_score;
                }
            }
        }

        // 生成改进建议
        let improvement_suggestions = self.generate_improvement_suggestions(&issues, action_type);

        // 判断是否需要重试
        let should_retry = self.should_retry(
            &ReflectionResult {
                action_type: action_type.to_string(),
                success,
                score,
                issues: issues.clone(),
                improvement_suggestions: improvement_suggestions.clone(),
                should_retry: false,
                retry_strategy: None,
            },
            &self.config,
        );

        let retry_strategy = if should_retry {
            Some(self.determine_retry_strategy(&issues, action_type))
        } else {
            None
        };

        ReflectionResult {
            action_type: action_type.to_string(),
            success,
            score,
            issues,
            improvement_suggestions,
            should_retry,
            retry_strategy,
        }
    }

    fn generate_improvement_suggestions(
        &self,
        issues: &[ActionIssue],
        action_type: &str,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        for issue in issues {
            match issue.category.as_str() {
                "执行错误" => {
                    suggestions.push(format!("解决执行错误：{}", issue.description));
                    suggestions.push("检查输入参数是否符合要求".to_string());
                }
                "质量不足" => {
                    suggestions.push(format!(
                        "提升{}质量，目标评分 >= {:.1}",
                        action_type, self.config.score_threshold
                    ));
                    suggestions.push("参考最佳实践和行业标准".to_string());
                }
                "输出格式" => {
                    suggestions.push("确保输出格式符合规范".to_string());
                    suggestions.push("添加必要的验证和边界检查".to_string());
                }
                _ => {
                    suggestions.push(format!("改进{}：{}", issue.category, issue.description));
                }
            }
        }

        if suggestions.is_empty() {
            suggestions.push("执行良好，继续保持".to_string());
        }

        suggestions
    }

    fn determine_retry_strategy(&self, issues: &[ActionIssue], action_type: &str) -> String {
        let critical_count = issues.iter().filter(|i| i.severity == "critical").count();

        if critical_count > 0 {
            return "立即停止，升级到人工处理".to_string();
        }

        let high_count = issues.iter().filter(|i| i.severity == "high").count();
        if high_count > 0 {
            return "修改后重试".to_string();
        }

        format!("优化{}后继续执行", action_type)
    }
}

impl Reflector for ActionReviewReflector {
    fn reflect(&self, result: &Value, action_type: &str) -> Result<ReflectionResult, String> {
        Ok(self.review_action(result, action_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_review_success() {
        let reflector = ActionReviewReflector::with_default();
        let result = json!({
            "status": "success",
            "quality_score": 85.0
        });

        let reflection = reflector.review_action(&result, "test_action");

        assert!(reflection.success);
        assert_eq!(reflection.score, 85.0);
        assert!(!reflection.should_retry);
    }

    #[test]
    fn test_review_failure() {
        let reflector = ActionReviewReflector::with_default();
        let result = json!({
            "status": "error",
            "error": "参数错误"
        });

        let reflection = reflector.review_action(&result, "test_action");

        assert!(!reflection.success);
        assert_eq!(reflection.score, 0.0);
        assert!(reflection.should_retry);
        assert_eq!(reflection.issues.len(), 1);
        assert_eq!(reflection.issues[0].category, "执行错误");
    }

    #[test]
    fn test_review_below_threshold() {
        let reflector = ActionReviewReflector::with_default();
        let result = json!({
            "status": "success",
            "quality_score": 65.0
        });

        let reflection = reflector.review_action(&result, "test_action");

        assert!(reflection.success);
        assert_eq!(reflection.score, 65.0);
        assert!(reflection.should_retry);
        assert!(reflection.retry_strategy.is_some());
    }

    #[test]
    fn test_improvement_suggestions() {
        let reflector = ActionReviewReflector::with_default();
        let issues = vec![ActionIssue {
            category: "执行错误".to_string(),
            severity: "critical".to_string(),
            description: "参数缺失".to_string(),
            location: None,
        }];

        let suggestions = reflector.generate_improvement_suggestions(&issues, "test_action");

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("参数缺失")));
    }
}
