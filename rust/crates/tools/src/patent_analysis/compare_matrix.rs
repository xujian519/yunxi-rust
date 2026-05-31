//! 对比矩阵工具包装层

use serde::Deserialize;
use serde_json::Value;

use patent_domain::compare::{
    build_feature_matrix, ipc_alignment, CompareFeature, FeatureMatcher, StructuredDiff,
};

/// 对比矩阵输入
#[derive(Debug, Deserialize)]
pub struct CompareMatrixInput {
    /// 目标发明文档
    pub target: TargetDocument,
    /// 现有技术文档
    pub prior_art: TargetDocument,
}

/// 文档结构
#[derive(Debug, Deserialize)]
pub struct TargetDocument {
    /// IPC 分类号
    #[serde(default)]
    pub ipc_codes: Vec<String>,
    /// 技术特征列表
    #[serde(default)]
    pub features: Vec<FeatureInput>,
}

/// 特征输入
#[derive(Debug, Deserialize)]
pub struct FeatureInput {
    /// 特征 ID
    pub id: String,
    /// 特征描述
    pub description: String,
}

/// 构建特征矩阵
pub fn build_compare_matrix(input: CompareMatrixInput) -> Result<Value, String> {
    let target_features: Vec<CompareFeature> = input
        .target
        .features
        .into_iter()
        .map(|f| CompareFeature {
            id: f.id,
            description: f.description,
        })
        .collect();

    let prior_features: Vec<CompareFeature> = input
        .prior_art
        .features
        .into_iter()
        .map(|f| CompareFeature {
            id: f.id,
            description: f.description,
        })
        .collect();

    let feature_matrix = build_feature_matrix(&target_features, &prior_features);
    let ipc_align = ipc_alignment(&input.target.ipc_codes, &input.prior_art.ipc_codes);

    let distinguishing_features: Vec<_> = feature_matrix
        .target_only
        .iter()
        .filter(|f| !f.is_empty())
        .cloned()
        .collect();

    let summary = if distinguishing_features.is_empty() {
        "两篇文档特征基本一致，无明显区别特征".to_string()
    } else {
        format!(
            "目标发明具有 {} 个区别特征：{}",
            distinguishing_features.len(),
            distinguishing_features.join(", ")
        )
    };

    let diff = StructuredDiff {
        feature_matrix,
        ipc_alignment: ipc_align,
        target_ipc: input.target.ipc_codes,
        prior_ipc: input.prior_art.ipc_codes,
        distinguishing_features,
        summary,
    };

    serde_json::to_value(diff).map_err(|e| e.to_string())
}

/// 特征匹配（侵权分析）
pub fn feature_match_analysis(input: CompareMatrixInput) -> Result<Value, String> {
    let target_features: Vec<CompareFeature> = input
        .target
        .features
        .into_iter()
        .map(|f| CompareFeature {
            id: f.id,
            description: f.description,
        })
        .collect();

    let prior_features: Vec<CompareFeature> = input
        .prior_art
        .features
        .into_iter()
        .map(|f| CompareFeature {
            id: f.id,
            description: f.description,
        })
        .collect();

    let result = FeatureMatcher::compare(&target_features, &prior_features);

    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_compare_matrix() {
        let input = CompareMatrixInput {
            target: TargetDocument {
                ipc_codes: vec!["G06F 3/01".into()],
                features: vec![
                    FeatureInput {
                        id: "f1".into(),
                        description: "包含传感器模块".into(),
                    },
                    FeatureInput {
                        id: "f2".into(),
                        description: "包含处理器模块".into(),
                    },
                ],
            },
            prior_art: TargetDocument {
                ipc_codes: vec!["G06F 3/01".into()],
                features: vec![
                    FeatureInput {
                        id: "p1".into(),
                        description: "包含传感器单元".into(),
                    },
                    FeatureInput {
                        id: "p2".into(),
                        description: "包含控制器".into(),
                    },
                ],
            },
        };

        let result = build_compare_matrix(input).unwrap();
        assert!(result["summary"].is_string());
        assert!(!result["featureMatrix"]["cells"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_feature_match_analysis() {
        let input = CompareMatrixInput {
            target: TargetDocument {
                ipc_codes: vec![],
                features: vec![
                    FeatureInput {
                        id: "f1".into(),
                        description: "包含传感器模块".into(),
                    },
                    FeatureInput {
                        id: "f2".into(),
                        description: "包含处理器模块".into(),
                    },
                ],
            },
            prior_art: TargetDocument {
                ipc_codes: vec![],
                features: vec![
                    FeatureInput {
                        id: "p1".into(),
                        description: "包含传感器模块".into(),
                    },
                    FeatureInput {
                        id: "p2".into(),
                        description: "包含控制器".into(),
                    },
                ],
            },
        };

        let result = feature_match_analysis(input).unwrap();
        assert!(result["coverageRatio"].as_f64().unwrap() >= 0.0);
        assert!(!result["exactMatches"].as_array().unwrap().is_empty());
    }
}
