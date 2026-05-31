//! 假设管理器
//!
//! 管理推理过程中的假设生成、去重和置信度更新。

/// 推理假设
#[derive(Debug, Clone, serde::Serialize)]
pub struct Hypothesis {
    pub id: usize,
    pub claim: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub generation_round: usize,
}

/// 假设管理器
pub struct HypothesisManager {
    hypotheses: Vec<Hypothesis>,
    next_id: usize,
    max_hypotheses: usize,
}

impl HypothesisManager {
    pub fn new(max_hypotheses: usize) -> Self {
        Self {
            hypotheses: Vec::new(),
            next_id: 1,
            max_hypotheses,
        }
    }

    /// 添加假设
    pub fn add(&mut self, claim: String, confidence: f64, round: usize) -> Option<usize> {
        if self.hypotheses.len() >= self.max_hypotheses {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;
        self.hypotheses.push(Hypothesis {
            id,
            claim,
            confidence,
            evidence: vec![],
            generation_round: round,
        });
        Some(id)
    }

    /// 为假设添加证据
    pub fn add_evidence(&mut self, id: usize, evidence: String) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.id == id) {
            h.evidence.push(evidence);
        }
    }

    /// 更新假设置信度
    pub fn update_confidence(&mut self, id: usize, new_confidence: f64) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.id == id) {
            h.confidence = new_confidence;
        }
    }

    /// 获取按置信度排序的假设
    pub fn ranked(&self) -> Vec<&Hypothesis> {
        let mut sorted: Vec<_> = self.hypotheses.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// 获取最佳假设
    pub fn best(&self) -> Option<&Hypothesis> {
        self.hypotheses.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// 检测重复假设（简化：基于 claim 文本完全匹配）
    pub fn is_duplicate(&self, claim: &str) -> bool {
        self.hypotheses.iter().any(|h| h.claim == claim)
    }

    /// 当前假设数量
    pub fn len(&self) -> usize {
        self.hypotheses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hypotheses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_rank() {
        let mut mgr = HypothesisManager::new(5);
        mgr.add("新颖".into(), 0.8, 1);
        mgr.add("不新颖".into(), 0.6, 1);
        mgr.add("部分新颖".into(), 0.9, 2);

        let ranked = mgr.ranked();
        assert_eq!(ranked[0].claim, "部分新颖");
        assert_eq!(ranked[2].claim, "不新颖");
    }

    #[test]
    fn test_max_limit() {
        let mut mgr = HypothesisManager::new(2);
        assert!(mgr.add("a".into(), 0.5, 1).is_some());
        assert!(mgr.add("b".into(), 0.5, 1).is_some());
        assert!(mgr.add("c".into(), 0.5, 1).is_none());
    }

    #[test]
    fn test_duplicate_detection() {
        let mut mgr = HypothesisManager::new(5);
        mgr.add("新颖".into(), 0.8, 1);
        assert!(mgr.is_duplicate("新颖"));
        assert!(!mgr.is_duplicate("创造性"));
    }

    #[test]
    fn test_evidence_and_confidence() {
        let mut mgr = HypothesisManager::new(5);
        let id = mgr.add("test".into(), 0.5, 1).unwrap();
        mgr.add_evidence(id, "证据1".into());
        mgr.update_confidence(id, 0.9);

        let h = mgr.best().unwrap();
        assert_eq!(h.id, id);
        assert_eq!(h.confidence, 0.9);
        assert_eq!(h.evidence.len(), 1);
    }
}
