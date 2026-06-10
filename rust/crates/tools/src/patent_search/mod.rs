//! 专利检索工具集

mod query_builder;
mod search_tools;
mod synonym;

pub use query_builder::*;
pub use search_tools::*;
pub use synonym::*;

pub mod retrieval;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synonym_lookup() {
        let input = SynonymSearchInput {
            operation: "lookup".to_string(),
            term: Some("机器学习".to_string()),
            terms: None,
            domain: None,
            accuracy: None,
            field: None,
            exclusions: None,
        };

        let result = synonym_search(input).unwrap();
        assert!(result["found"].as_bool().unwrap());
        assert_eq!(result["entry"]["chinese"], "机器学习");
        assert_eq!(result["entry"]["domain"], "AI");
    }

    #[test]
    fn test_synonym_expand() {
        let input = SynonymSearchInput {
            operation: "expand".to_string(),
            term: None,
            terms: Some(vec!["机器学习".to_string(), "神经网络".to_string()]),
            domain: None,
            accuracy: None,
            field: None,
            exclusions: None,
        };

        let result = synonym_search(input).unwrap();
        assert!(result["expanded_synonyms"].as_array().unwrap().len() > 2);
    }

    #[test]
    fn test_detect_domain() {
        let input = SynonymSearchInput {
            operation: "detect_domain".to_string(),
            term: Some("该专利涉及深度学习和神经网络在计算机视觉中的应用".to_string()),
            terms: None,
            domain: None,
            accuracy: None,
            field: None,
            exclusions: None,
        };

        let result = synonym_search(input).unwrap();
        assert_eq!(result["primary_domain"].as_str().unwrap(), "AI");
    }

    #[test]
    fn test_search_query_builder() {
        let input = SearchQueryBuilderInput {
            keywords: vec!["机器学习".to_string(), "图像识别".to_string()],
            ipc_codes: Some(vec!["G06N".to_string()]),
            technical_field: Some("人工智能".to_string()),
            technical_problem: Some("识别精度低".to_string()),
            domain_strategy: Some("focused".to_string()),
        };

        let result = search_query_builder(input).unwrap();
        assert_eq!(result["stages"].as_array().unwrap().len(), 3);
        assert!(result["stages"][0]["query"]
            .as_str()
            .unwrap()
            .contains("机器学习"));
    }

    #[test]
    fn test_patent_search_returns_status() {
        let input = PatentSearchInput {
            query: "深度学习".to_string(),
            search_type: Some("keyword".to_string()),
            limit: Some(5),
            offset: Some(0),
        };

        let result = patent_search(input).unwrap();
        assert!(result["status"].is_string());
        assert_eq!(result["query"], "深度学习");
    }

    #[test]
    fn test_iterative_search() {
        let input = IterativeSearchInput {
            query: "神经网络 图像识别".to_string(),
            max_iterations: Some(2),
            search_type: Some("patent".to_string()),
            width: Some(3),
        };

        let result = iterative_search(input).unwrap();
        assert_eq!(result["mode"], "rule_based");
        assert!(result["search_plan"].as_array().unwrap().len() <= 2);
    }
}
