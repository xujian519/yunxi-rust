//! 云熙专利非 LLM 对比内核。
//!
//! 提供特征矩阵构建、结构化 diff、IPC class 级分类等能力，
//! 不依赖 LLM，纯规则/词法计算。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

// ==================== IPC 分类 ====================

/// IPC 部（section）分类器。
pub struct IpcClassifier {
    section_keywords: HashMap<String, Vec<String>>,
}

impl Default for IpcClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcClassifier {
    #[must_use]
    pub fn new() -> Self {
        let mut section_keywords = HashMap::new();
        section_keywords.insert(
            "A".into(),
            vec!["农业".into(), "食品".into(), "医药".into(), "生活".into()],
        );
        section_keywords.insert(
            "B".into(),
            vec!["加工".into(), "运输".into(), "包装".into(), "机床".into()],
        );
        section_keywords.insert(
            "C".into(),
            vec![
                "化学".into(),
                "冶金".into(),
                "聚合物".into(),
                "催化剂".into(),
            ],
        );
        section_keywords.insert(
            "G".into(),
            vec!["计算".into(), "测量".into(), "控制".into(), "物理".into()],
        );
        section_keywords.insert(
            "H".into(),
            vec![
                "电".into(),
                "通信".into(),
                "半导体".into(),
                "电路".into(),
                "电池".into(),
            ],
        );
        section_keywords.insert(
            "F".into(),
            vec!["发动机".into(), "泵".into(), "齿轮".into(), "机械".into()],
        );
        Self { section_keywords }
    }

    /// 返回按相关度排序的 IPC section 列表（如 `H`、`G`）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn classify_sections(&self, text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        let mut scored: Vec<(String, f32)> = self
            .section_keywords
            .iter()
            .map(|(section, kws)| {
                let hits = kws.iter().filter(|k| lower.contains(k.as_str())).count();
                let score = if kws.is_empty() {
                    0.0
                } else {
                    hits as f32 / kws.len() as f32
                };
                (section.clone(), score)
            })
            .filter(|(_, s)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(s, _)| s).collect()
    }
}

// ==================== 特征矩阵 / 结构化 Diff ====================

/// 单条技术特征（发明/现有技术通用）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompareFeature {
    pub id: String,
    pub description: String,
}

/// 对比一侧文档摘要。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompareDocument {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub abstract_text: String,
    #[serde(default)]
    pub claims: Vec<String>,
    #[serde(default)]
    pub ipc_codes: Vec<String>,
    #[serde(default)]
    pub features: Vec<CompareFeature>,
}

/// 特征对齐单元格。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatrixCell {
    pub target_index: usize,
    pub prior_index: usize,
    pub lexical_score: f64,
    pub matched: bool,
}

/// 特征矩阵（非 LLM）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatrix {
    pub cells: Vec<FeatureMatrixCell>,
    pub target_only: Vec<String>,
    pub prior_only: Vec<String>,
    pub overlap_ratio: f64,
}

/// 结构化 diff 摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredDiff {
    pub feature_matrix: FeatureMatrix,
    pub ipc_alignment: f64,
    pub target_ipc: Vec<String>,
    pub prior_ipc: Vec<String>,
    pub distinguishing_features: Vec<String>,
    pub summary: String,
}

/// 构建特征矩阵（词法 Jaccard + 子串命中）。
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn build_feature_matrix(target: &[CompareFeature], prior: &[CompareFeature]) -> FeatureMatrix {
    let mut cells = Vec::new();
    let mut matched_target = HashSet::new();
    let mut matched_prior = HashSet::new();

    for (ti, tf) in target.iter().enumerate() {
        for (pi, pf) in prior.iter().enumerate() {
            let score = lexical_similarity(&tf.description, &pf.description);
            let matched = score >= 0.45;
            if matched {
                matched_target.insert(ti);
                matched_prior.insert(pi);
            }
            cells.push(FeatureMatrixCell {
                target_index: ti,
                prior_index: pi,
                lexical_score: score,
                matched,
            });
        }
    }

    let target_only: Vec<String> = target
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_target.contains(i))
        .map(|(_, f)| f.description.clone())
        .collect();

    let prior_only: Vec<String> = prior
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_prior.contains(i))
        .map(|(_, f)| f.description.clone())
        .collect();

    let overlap = if target.is_empty() {
        0.0
    } else {
        matched_target.len() as f64 / target.len() as f64
    };

    FeatureMatrix {
        cells,
        target_only,
        prior_only,
        overlap_ratio: overlap,
    }
}

/// 结构化对比（含 IPC 对齐）。
#[must_use]
pub fn structured_diff(target: &CompareDocument, prior: &CompareDocument) -> StructuredDiff {
    static CLASSIFIER: std::sync::OnceLock<IpcClassifier> = std::sync::OnceLock::new();
    let classifier = CLASSIFIER.get_or_init(IpcClassifier::new);
    let target_ipc = if target.ipc_codes.is_empty() {
        classifier.classify_sections(&format!("{} {}", target.title, target.abstract_text))
    } else {
        target.ipc_codes.clone()
    };
    let prior_ipc = if prior.ipc_codes.is_empty() {
        classifier.classify_sections(&format!("{} {}", prior.title, prior.abstract_text))
    } else {
        prior.ipc_codes.clone()
    };

    let target_features = if target.features.is_empty() {
        claims_to_features(&target.claims, "t")
    } else {
        target.features.clone()
    };
    let prior_features = if prior.features.is_empty() {
        claims_to_features(&prior.claims, "p")
    } else {
        prior.features.clone()
    };

    let matrix = build_feature_matrix(&target_features, &prior_features);
    let ipc_alignment = ipc_alignment_score(&target_ipc, &prior_ipc);
    let distinguishing = matrix.target_only.clone();

    let summary = format!(
        "特征重叠率 {:.0}%，IPC 对齐 {:.0}%，区别特征 {} 项",
        matrix.overlap_ratio * 100.0,
        ipc_alignment * 100.0,
        distinguishing.len()
    );

    StructuredDiff {
        feature_matrix: matrix,
        ipc_alignment,
        target_ipc,
        prior_ipc,
        distinguishing_features: distinguishing,
        summary,
    }
}

fn claims_to_features(claims: &[String], prefix: &str) -> Vec<CompareFeature> {
    claims
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let desc = c.trim();
            if desc.len() < 4 {
                return None;
            }
            Some(CompareFeature {
                id: format!("{prefix}{i}"),
                description: desc.to_string(),
            })
        })
        .collect()
}

#[allow(clippy::cast_precision_loss)]
fn ipc_alignment_score(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let set_a: HashSet<_> = a.iter().collect();
    let overlap = b.iter().filter(|x| set_a.contains(x)).count();
    overlap as f64 / a.len().max(b.len()) as f64
}

#[allow(clippy::cast_precision_loss)]
fn lexical_similarity(a: &str, b: &str) -> f64 {
    let a_l = a.to_lowercase();
    let b_l = b.to_lowercase();
    if a_l == b_l {
        return 1.0;
    }
    if a_l.contains(b_l.as_str()) || b_l.contains(a_l.as_str()) {
        return 0.75;
    }
    let grams_a = char_bigrams(&a_l);
    let grams_b = char_bigrams(&b_l);
    if grams_a.is_empty() || grams_b.is_empty() {
        return 0.0;
    }
    let inter = grams_a.intersection(&grams_b).count();
    let union = grams_a.union(&grams_b).count();
    inter as f64 / union as f64
}

fn char_bigrams(s: &str) -> HashSet<(char, char)> {
    let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut set = HashSet::new();
    for w in chars.windows(2) {
        set.insert((w[0], w[1]));
    }
    set
}

// ==================== 工具入口 ====================

/// 工具输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentCompareInput {
    /// 操作模式：`"diff"` 结构化对比（默认），`"matrix"` 仅特征矩阵，`"ipc"` IPC 分类。
    #[serde(default = "default_mode")]
    pub mode: String,
    /// 目标发明文档（diff / matrix 模式）。
    #[serde(default)]
    pub target: Option<CompareDocument>,
    /// 现有技术文档（diff / matrix 模式）。
    #[serde(default)]
    pub prior_art: Option<CompareDocument>,
    /// 待分类文本（ipc 模式）。
    #[serde(default)]
    pub text: Option<String>,
}

fn default_mode() -> String {
    "diff".into()
}

/// 工具输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PatentCompareOutput {
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    structured_diff: Option<StructuredDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feature_matrix: Option<FeatureMatrix>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipc_sections: Option<Vec<String>>,
}

/// 执行专利非 LLM 对比。
///
/// # Errors
///
/// 当输入缺少必要字段或序列化失败时返回错误字符串。
pub fn execute_patent_compare(input: &PatentCompareInput) -> Result<Value, String> {
    static CLASSIFIER: std::sync::OnceLock<IpcClassifier> = std::sync::OnceLock::new();

    // diff / matrix 模式需要 target 和 priorArt 字段
    if (input.mode == "diff" || input.mode == "matrix")
        && (input.target.is_none() || input.prior_art.is_none())
    {
        return Err(format!(
            "模式 '{}' 需要 target 和 priorArt 字段",
            input.mode
        ));
    }

    let output = match input.mode.as_str() {
        "matrix" => {
            let target = input.target.clone().unwrap_or_default();
            let prior = input.prior_art.clone().unwrap_or_default();
            let matrix = build_feature_matrix(&target.features, &prior.features);
            PatentCompareOutput {
                mode: "matrix".into(),
                structured_diff: None,
                feature_matrix: Some(matrix),
                ipc_sections: None,
            }
        }
        "ipc" => {
            let text = input.text.clone().unwrap_or_default();
            let classifier = CLASSIFIER.get_or_init(IpcClassifier::new);
            let sections = classifier.classify_sections(&text);
            PatentCompareOutput {
                mode: "ipc".into(),
                structured_diff: None,
                feature_matrix: None,
                ipc_sections: Some(sections),
            }
        }
        // 默认 "diff" 模式
        _ => {
            let target = input.target.clone().unwrap_or_default();
            let prior = input.prior_art.clone().unwrap_or_default();
            let diff = structured_diff(&target, &prior);
            PatentCompareOutput {
                mode: "diff".into(),
                structured_diff: Some(diff),
                feature_matrix: None,
                ipc_sections: None,
            }
        }
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

// ==================== 测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    // --- IPC ---

    #[test]
    fn classifies_semiconductor_as_h() {
        let c = IpcClassifier::new();
        let sections = c.classify_sections("半导体通信电路装置");
        assert_eq!(sections.first().map(String::as_str), Some("H"));
    }

    // --- Matrix ---

    #[test]
    fn matrix_detects_overlap_and_diff() {
        let target = vec![
            CompareFeature {
                id: "t0".into(),
                description: "包括处理器模块与存储单元".into(),
            },
            CompareFeature {
                id: "t1".into(),
                description: "采用神经网络推理引擎".into(),
            },
        ];
        let prior = vec![CompareFeature {
            id: "p0".into(),
            description: "包括处理器模块".into(),
        }];
        let m = build_feature_matrix(&target, &prior);
        assert!(m.overlap_ratio > 0.0);
        assert!(!m.target_only.is_empty());
    }

    #[test]
    fn structured_diff_ipc_and_features() {
        let target = CompareDocument {
            title: "半导体通信电路".into(),
            abstract_text: "本发明涉及半导体通信".into(),
            claims: vec!["一种通信电路，包括处理器".into()],
            ..Default::default()
        };
        let prior = CompareDocument {
            title: "传统通信装置".into(),
            abstract_text: "现有电学通信方案".into(),
            claims: vec!["一种装置，包括处理器".into()],
            ..Default::default()
        };
        let diff = structured_diff(&target, &prior);
        assert!(diff.ipc_alignment >= 0.0);
        assert!(!diff.summary.is_empty());
    }

    // --- Tool ---

    #[test]
    fn test_diff_mode() {
        let result = execute_patent_compare(&PatentCompareInput {
            mode: "diff".into(),
            target: Some(CompareDocument {
                title: "半导体通信电路".into(),
                abstract_text: "本发明涉及半导体通信".into(),
                claims: vec!["一种通信电路，包括处理器".into()],
                ..Default::default()
            }),
            prior_art: Some(CompareDocument {
                title: "传统通信装置".into(),
                abstract_text: "现有电学通信方案".into(),
                claims: vec!["一种装置，包括处理器".into()],
                ..Default::default()
            }),
            text: None,
        });
        let value = result.unwrap();
        assert_eq!(value["mode"], "diff");
        assert!(value["structuredDiff"].is_object());
    }

    #[test]
    fn test_ipc_mode() {
        let result = execute_patent_compare(&PatentCompareInput {
            mode: "ipc".into(),
            target: None,
            prior_art: None,
            text: Some("一种基于半导体通信电路的计算装置".into()),
        });
        let value = result.unwrap();
        assert_eq!(value["mode"], "ipc");
        let sections = value["ipcSections"].as_array().unwrap();
        assert!(!sections.is_empty());
    }

    #[test]
    fn test_matrix_mode() {
        let result = execute_patent_compare(&PatentCompareInput {
            mode: "matrix".into(),
            target: Some(CompareDocument {
                features: vec![CompareFeature {
                    id: "t0".into(),
                    description: "包括处理器模块与存储单元".into(),
                }],
                ..Default::default()
            }),
            prior_art: Some(CompareDocument {
                features: vec![CompareFeature {
                    id: "p0".into(),
                    description: "包括处理器模块".into(),
                }],
                ..Default::default()
            }),
            text: None,
        });
        let value = result.unwrap();
        assert_eq!(value["mode"], "matrix");
        assert!(value["featureMatrix"].is_object());
    }

    #[test]
    fn test_default_mode_is_diff() {
        let result = execute_patent_compare(&PatentCompareInput {
            mode: "diff".into(),
            target: Some(CompareDocument {
                title: "测试发明".into(),
                ..Default::default()
            }),
            prior_art: Some(CompareDocument {
                title: "测试现有技术".into(),
                ..Default::default()
            }),
            text: None,
        });
        let value = result.unwrap();
        assert_eq!(value["mode"], "diff");
    }

    /// diff 模式缺少 target/priorArt 时应报错。
    #[test]
    fn test_diff_mode_requires_target_and_prior_art() {
        let result = execute_patent_compare(&PatentCompareInput {
            mode: "diff".into(),
            target: Some(CompareDocument {
                title: "仅目标".into(),
                ..Default::default()
            }),
            prior_art: None,
            text: None,
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("priorArt"),
            "错误信息应提及缺少 priorArt: {err}"
        );
    }

    /// matrix 模式缺少 target/priorArt 时应报错。
    #[test]
    fn test_matrix_mode_requires_target_and_prior_art() {
        let result = execute_patent_compare(&PatentCompareInput {
            mode: "matrix".into(),
            target: None,
            prior_art: None,
            text: None,
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("target") || err.contains("priorArt"),
            "错误信息应提及缺少字段: {err}"
        );
    }
}
