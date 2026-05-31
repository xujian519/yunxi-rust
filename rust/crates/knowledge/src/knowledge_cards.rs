//! 知识卡片管理
//!
//! 加载和管理专利知识卡片，支持按概念、质量分和关键词检索。

use crate::types::KnowledgeCard;
use std::path::Path;

/// 知识卡片索引
pub struct CardIndex {
    cards: Vec<KnowledgeCard>,
    base_dir: String,
}

impl CardIndex {
    /// 从 card-index.json 加载卡片索引
    pub fn load(index_path: &str) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(index_path).map_err(|e| format!("读取卡片索引失败: {e}"))?;

        let raw: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析卡片索引失败: {e}"))?;

        let cards_array = raw
            .get("cards")
            .and_then(|v| v.as_array())
            .ok_or("卡片索引缺少 cards 字段")?;

        let base_dir = Path::new(index_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".into());

        let mut cards = Vec::new();
        for card_val in cards_array {
            match serde_json::from_value::<KnowledgeCard>(card_val.clone()) {
                Ok(mut card) => {
                    // 将路径中的绝对前缀替换为相对路径
                    let filename = Path::new(&card.file_path)
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();
                    card.file_path = filename;
                    cards.push(card);
                }
                Err(e) => eprintln!("Warning: skipping invalid card entry: {e}"),
            }
        }

        Ok(Self { cards, base_dir })
    }

    /// 获取卡片总数
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// 获取全部卡片（不含正文）
    pub fn all(&self) -> &[KnowledgeCard] {
        &self.cards
    }

    /// 按概念关键词搜索
    pub fn search_by_keyword(&self, keyword: &str, limit: usize) -> Vec<KnowledgeCard> {
        let kw_lower = keyword.to_lowercase();
        let mut results: Vec<KnowledgeCard> = self
            .cards
            .iter()
            .filter(|c| {
                c.title.to_lowercase().contains(&kw_lower)
                    || c.concept.to_lowercase().contains(&kw_lower)
                    || c.related_concepts
                        .iter()
                        .any(|rc| rc.to_lowercase().contains(&kw_lower))
            })
            .cloned()
            .collect();

        // 按质量分降序排列
        results.sort_by(|a, b| {
            b.quality
                .partial_cmp(&a.quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    /// 按质量分筛选（只返回 quality >= threshold 的卡片）
    pub fn filter_by_quality(&self, threshold: f64, limit: usize) -> Vec<&KnowledgeCard> {
        let mut results: Vec<&KnowledgeCard> = self
            .cards
            .iter()
            .filter(|c| c.quality >= threshold)
            .collect();
        results.sort_by(|a, b| {
            b.quality
                .partial_cmp(&a.quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    /// 加载卡片正文内容
    pub fn load_content(&self, card: &mut KnowledgeCard) -> Result<(), String> {
        if !card.content.is_empty() {
            return Ok(());
        }
        let path = format!("{}/{}", self.base_dir, card.file_path);
        card.content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取卡片内容失败 {}: {e}", card.file_path))?;
        Ok(())
    }

    /// 按关键词搜索并加载正文
    pub fn search_with_content(&self, keyword: &str, limit: usize) -> Vec<KnowledgeCard> {
        let mut results = self.search_by_keyword(keyword, limit);
        for card in &mut results {
            let _ = self.load_content(card);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn card_index_path() -> Option<String> {
        let path = "../../../assets/knowledge/cards/card-index.json";
        if Path::new(path).exists() {
            Some(path.to_string())
        } else {
            None
        }
    }

    #[test]
    fn test_load_card_index() {
        let Some(path) = card_index_path() else {
            eprintln!("Skipping test: card-index.json not found");
            return;
        };
        let index = CardIndex::load(&path).unwrap();
        assert!(!index.is_empty(), "should have loaded cards");
    }

    #[test]
    fn test_search_cards() {
        let Some(path) = card_index_path() else {
            eprintln!("Skipping test: card-index.json not found");
            return;
        };
        let index = CardIndex::load(&path).unwrap();
        let results = index.search_by_keyword("创造性", 5);
        // 可能找到也可能找不到取决于卡片内容
        if !results.is_empty() {
            assert!(results.len() <= 5);
        }
    }

    #[test]
    fn test_filter_by_quality() {
        let Some(path) = card_index_path() else {
            eprintln!("Skipping test: card-index.json not found");
            return;
        };
        let index = CardIndex::load(&path).unwrap();
        let high_quality = index.filter_by_quality(0.8, 10);
        // 只检查不崩溃
        assert!(high_quality.len() <= 10);
    }
}
