//! 记忆相关性评分
//!
//! 基于词频加权、内容长度归一化、标签匹配和时效性的综合评分。

use crate::types::MemoryEntry;

/// 计算查询与记忆条目的相关性分数
pub fn score_relevance(query: &str, entry: &MemoryEntry) -> f64 {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|t| t.len() > 1)
        .collect();

    if query_terms.is_empty() {
        return 0.0;
    }

    let content_lower = entry.content.to_lowercase();
    let tags_lower: Vec<String> = entry.meta.tags.iter().map(|t| t.to_lowercase()).collect();

    // 词频加权：每个查询词在内容中的出现次数（上限3次）
    let mut total_hits = 0usize;
    for term in &query_terms {
        let count = content_lower.matches(*term).count().min(3);
        total_hits += count;
    }

    if total_hits == 0 {
        // 仅标签匹配时给予较低分数
        let tag_matches = query_terms
            .iter()
            .filter(|qt| tags_lower.iter().any(|t| t.contains(*qt)))
            .count();
        if tag_matches == 0 {
            return 0.0;
        }
        return (tag_matches as f64 / query_terms.len() as f64 * 0.5).min(1.0);
    }

    // 内容长度归一化：防止长文档仅因字数多而占优
    let content_len_norm = (content_lower.chars().count() as f64).ln().max(1.0);
    let raw_score = total_hits as f64 / (query_terms.len() as f64 * content_len_norm);

    // 标签逐项加分：每个匹配标签 +0.1，上限 0.3
    let tag_matches = query_terms
        .iter()
        .filter(|qt| tags_lower.iter().any(|t| t.contains(*qt)))
        .count();
    let tag_bonus = (tag_matches as f64 * 0.1).min(0.3);

    // 时效性加权：7 天内更新的条目获得 0.05 加分
    let recency_bonus = recency_bonus(&entry.meta.updated_at);

    (raw_score + tag_bonus + recency_bonus).min(1.0)
}

fn recency_bonus(updated_at: &str) -> f64 {
    if updated_at.is_empty() {
        return 0.0;
    }
    // 兼容 "2025-01-01T00:00:00" 和 "2025-01-01T00:00:00+00:00" 两种格式
    let dt = updated_at
        .parse::<chrono::DateTime<chrono::Utc>>()
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(updated_at, "%Y-%m-%dT%H:%M:%S").map(|nd| {
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(nd, chrono::Utc)
            })
        });
    dt.map(|dt| {
        let days = (chrono::Utc::now() - dt).num_days().max(0);
        if days <= 7 {
            0.05
        } else {
            0.0
        }
    })
    .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MemoryMeta, MemoryType};
    use std::path::PathBuf;

    fn make_entry(content: &str, tags: Vec<&str>, updated_at: &str) -> MemoryEntry {
        MemoryEntry {
            path: PathBuf::from("/tmp/test.md"),
            meta: MemoryMeta {
                memory_type: MemoryType::Project,
                tags: tags.into_iter().map(String::from).collect(),
                created_at: String::new(),
                updated_at: updated_at.to_string(),
            },
            content: content.to_string(),
        }
    }

    #[test]
    fn test_exact_match_scores_high() {
        let entry = make_entry("这是一个测试记忆条目", vec![], "");
        let score = score_relevance("测试记忆", &entry);
        assert!(score > 0.3, "expected score > 0.3, got {score}");
    }

    #[test]
    fn test_no_match_scores_zero() {
        let entry = make_entry("完全无关的内容", vec![], "");
        let score = score_relevance("xyzabc", &entry);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_tag_bonus_adds_to_score() {
        let no_tags = make_entry("基础内容", vec![], "");
        let with_tags = make_entry("基础内容", vec!["测试标签"], "");
        let score_no = score_relevance("测试标签", &no_tags);
        let score_yes = score_relevance("测试标签", &with_tags);
        assert!(
            score_yes > score_no,
            "tagged entry should score higher: {score_yes} vs {score_no}"
        );
    }

    #[test]
    fn test_content_length_normalization() {
        let short = make_entry("测试测试", vec![], "");
        let long = make_entry(&"测试".repeat(200), vec![], "");
        let score_short = score_relevance("测试", &short);
        let score_long = score_relevance("测试", &long);
        // 两者都匹配，但短文档不应被长文档完全压倒
        assert!(
            score_short >= score_long * 0.3,
            "short={score_short}, long={score_long}"
        );
    }

    #[test]
    fn test_recency_bonus() {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let old = "2020-01-01T00:00:00".to_string();
        let recent = make_entry("测试内容", vec![], &now);
        let outdated = make_entry("测试内容", vec![], &old);
        let score_recent = score_relevance("测试", &recent);
        let score_old = score_relevance("测试", &outdated);
        assert!(
            score_recent > score_old,
            "recent={score_recent}, old={score_old}"
        );
    }
}
