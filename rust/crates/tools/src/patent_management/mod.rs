//! 专利管理工具集 - 专利生命周期管理、模板库、商标分析等
//!
//! 提供专利管理核心功能，包括：
//! - 专利生命周期管理（增删改查、状态机、期限管理、费用管理）
//! - 专利文档模板库（审查意见答复、专利申请、无效宣告等）
//! - 商标可注册性分析（基于规则评分）
//! - 专利下载（单件/批量，存根实现）

mod lifecycle;
mod trademark;

pub use lifecycle::{execute_patent_manager, execute_template_library, PatentManagerInput};
pub use trademark::{
    execute_batch_patent_download, execute_patent_download, execute_trademark_analysis,
    BatchPatentDownloadInput, PatentDownloadInput, TrademarkAnalysisInput,
};

// Re-export TemplateLibraryInput which is used by lib.rs
pub use lifecycle::TemplateLibraryInput;

#[cfg(test)]
mod tests {
    use super::lifecycle::{PatentRecord, TemplateLibraryInput};
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_patent_manager_add_and_get() {
        let patent = PatentRecord {
            id: "CN123456".to_string(),
            title: "测试专利".to_string(),
            applicant: "测试公司".to_string(),
            inventor: "张三".to_string(),
            filing_date: "2024-01-01".to_string(),
            patent_type: "发明".to_string(),
            status: "draft".to_string(),
            notes: Some("测试备注".to_string()),
        };

        let add_input = PatentManagerInput {
            operation: "add".to_string(),
            patent_id: None,
            patent: Some(patent.clone()),
            new_status: None,
            deadline: None,
            fee: None,
        };

        execute_patent_manager(add_input).unwrap();

        let get_input = PatentManagerInput {
            operation: "get".to_string(),
            patent_id: Some("CN123456".to_string()),
            patent: None,
            new_status: None,
            deadline: None,
            fee: None,
        };

        let result = execute_patent_manager(get_input).unwrap();
        assert_eq!(result["patent"]["id"], "CN123456");
    }

    #[test]
    fn test_patent_status_transition() {
        let patent = PatentRecord {
            id: "CN789012".to_string(),
            title: "状态测试".to_string(),
            applicant: "测试公司".to_string(),
            inventor: "李四".to_string(),
            filing_date: "2024-01-01".to_string(),
            patent_type: "发明".to_string(),
            status: "draft".to_string(),
            notes: None,
        };

        let add_input = PatentManagerInput {
            operation: "add".to_string(),
            patent_id: None,
            patent: Some(patent),
            new_status: None,
            deadline: None,
            fee: None,
        };

        execute_patent_manager(add_input).unwrap();

        // 有效转换：draft -> filed
        let change_input = PatentManagerInput {
            operation: "change_status".to_string(),
            patent_id: Some("CN789012".to_string()),
            patent: None,
            new_status: Some("filed".to_string()),
            deadline: None,
            fee: None,
        };

        let result = execute_patent_manager(change_input);
        assert!(result.is_ok());

        // 无效转换：filed -> granted（跳过状态）
        let invalid_change_input = PatentManagerInput {
            operation: "change_status".to_string(),
            patent_id: Some("CN789012".to_string()),
            patent: None,
            new_status: Some("granted".to_string()),
            deadline: None,
            fee: None,
        };

        let result = execute_patent_manager(invalid_change_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_library_list() {
        let input = TemplateLibraryInput {
            operation: "list".to_string(),
            template_type: None,
            template_id: None,
            variables: None,
        };

        let result = execute_template_library(input).unwrap();
        assert_eq!(result["total"], 5);
        assert!(result["templates"].as_array().unwrap().len() == 5);
    }

    #[test]
    fn test_template_render() {
        let mut variables = HashMap::new();
        variables.insert("applicant_name".to_string(), "测试公司".to_string());
        variables.insert("application_number".to_string(), "CN2024123456".to_string());
        variables.insert("invention_title".to_string(), "测试发明".to_string());
        variables.insert("rejection_summary".to_string(), "无新颖性".to_string());
        variables.insert("response_arguments".to_string(), "具有创造性".to_string());
        variables.insert(
            "amendments_summary".to_string(),
            "修改权利要求1".to_string(),
        );
        variables.insert("conclusion".to_string(), "请求授权".to_string());

        let input = TemplateLibraryInput {
            operation: "render".to_string(),
            template_type: Some("oa-response".to_string()),
            template_id: None,
            variables: Some(variables),
        };

        let result = execute_template_library(input).unwrap();
        assert!(result["rendered"].as_str().unwrap().contains("测试公司"));
    }

    #[test]
    fn test_trademark_analysis() {
        let input = TrademarkAnalysisInput {
            trademark_name: "超级优质".to_string(),
            goods_services: Some("优质商品".to_string()),
            trademark_type: Some("word".to_string()),
        };

        let result = execute_trademark_analysis(input).unwrap();
        assert!(result["registrability_score"].as_f64().unwrap() < 0.5);
        assert_eq!(result["distinctiveness"], "低");
    }

    #[test]
    fn test_patent_download_stub() {
        let input = PatentDownloadInput {
            patent_id: "CN123456".to_string(),
            output_dir: Some("./downloads".to_string()),
            format: Some("pdf".to_string()),
        };

        let result = execute_patent_download(input).unwrap();
        assert_eq!(result["status"], "pending_configuration");
    }

    #[test]
    fn test_batch_patent_download_stub() {
        let input = BatchPatentDownloadInput {
            patent_ids: vec!["CN123456".to_string(), "CN789012".to_string()],
            output_dir: Some("./downloads".to_string()),
            format: Some("xml".to_string()),
        };

        let result = execute_batch_patent_download(input).unwrap();
        assert_eq!(result["total"], 2);
        assert_eq!(result["downloaded"], 0);
        assert_eq!(result["failed"], 2);
    }
}
