//! 专利领域基础模型定义

/// 权利要求类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClaimType {
    Independent,
    Dependent,
}

/// 特征类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureType {
    Element,
    Action,
    Parameter,
    Condition,
    Result,
}

/// 特征对应关系类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorrespondenceType {
    Exact,
    Equivalent,
    Different,
    Missing,
}

/// 解析后的特征
#[derive(Debug, Clone)]
pub struct ParsedFeature {
    pub id: String,
    pub description: String,
    pub feature_type: FeatureType,
    pub component: Option<String>,
    pub parameters: Vec<String>,
}

/// 解析后的权利要求
#[derive(Debug, Clone)]
pub struct ParsedClaim {
    pub claim_number: u32,
    pub claim_type: ClaimType,
    pub preamble: String,
    pub transition_word: String,
    pub body: String,
    pub features: Vec<ParsedFeature>,
    pub dependent_from: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_type_equality() {
        assert_eq!(ClaimType::Independent, ClaimType::Independent);
        assert_ne!(ClaimType::Independent, ClaimType::Dependent);
    }

    #[test]
    fn feature_type_variants() {
        let types = [
            FeatureType::Element,
            FeatureType::Action,
            FeatureType::Parameter,
            FeatureType::Condition,
            FeatureType::Result,
        ];
        for i in 0..types.len() {
            for j in 0..types.len() {
                if i == j {
                    assert_eq!(types[i], types[j]);
                } else {
                    assert_ne!(types[i], types[j]);
                }
            }
        }
    }

    #[test]
    fn correspondence_type_variants() {
        assert_eq!(CorrespondenceType::Exact, CorrespondenceType::Exact);
        assert_eq!(CorrespondenceType::Different, CorrespondenceType::Different);
        assert_ne!(CorrespondenceType::Exact, CorrespondenceType::Equivalent);
    }

    #[test]
    fn parsed_feature_construction() {
        let f = ParsedFeature {
            id: "f1".into(),
            description: "housing".into(),
            feature_type: FeatureType::Element,
            component: Some("frame".into()),
            parameters: vec!["aluminum".into()],
        };
        assert_eq!(f.id, "f1");
        assert_eq!(f.feature_type, FeatureType::Element);
        assert!(f.component.is_some());
        assert_eq!(f.parameters.len(), 1);
    }

    #[test]
    fn parsed_claim_construction() {
        let claim = ParsedClaim {
            claim_number: 1,
            claim_type: ClaimType::Independent,
            preamble: "A widget".into(),
            transition_word: "comprising".into(),
            body: "a housing and a sensor.".into(),
            features: vec![],
            dependent_from: None,
        };
        assert_eq!(claim.claim_number, 1);
        assert_eq!(claim.claim_type, ClaimType::Independent);
        assert!(claim.dependent_from.is_none());
    }
}
