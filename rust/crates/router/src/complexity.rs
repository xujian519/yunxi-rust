//! 任务复杂度评估器

use crate::types::Complexity;

/// 复杂度评估器
pub struct ComplexityAssessor;

impl ComplexityAssessor {
    pub fn new() -> Self {
        Self
    }

    /// 评估输入的任务复杂度
    pub fn assess(&self, input: &str) -> Complexity {
        // 复杂：需要多步骤、创造性工作、或跨领域协调
        let complex_verbs = [
            "撰写",
            "起草",
            "编写",
            "编制",
            "拟定",
            "申请",
            "申报",
            "提交",
            "答辩",
            "答复",
            "回复",
            "回复审查意见",
            "无效",
            "无效宣告",
            "无效答辩",
            "异议",
            "复审",
            "驳回复审",
            "全流程",
            "一站式",
            "代理",
            "代办",
            "布局",
            "挖掘",
            "规避设计",
            "侵权分析",
            "FTO",
            "自由实施",
            "许可谈判",
            "技术转移",
            "诉讼",
            "起诉",
            "应诉",
            "组合",
            "批量",
            "多件",
        ];

        // 中等：需要分析判断，但单一领域
        let medium_verbs = [
            "分析", "评估", "评价", "判断", "诊断", "检索", "搜索", "查询", "查新", "对比", "比较",
            "对标", "检查", "审查", "审核", "复核", "翻译", "校对", "监控", "预警", "跟踪", "监测",
            "分类", "聚类", "归纳", "总结", "摘要", "概括", "建议", "推荐", "改进", "优化", "完善",
            "修改", "修订", "补正", "解读", "解析",
        ];

        // 简单：信息查询或简单解释
        let simple_verbs = [
            "查看",
            "列出",
            "显示",
            "展示",
            "什么是",
            "什么叫",
            "如何理解",
            "如何定义",
            "解释",
            "说明",
            "介绍一下",
            "讲一下",
            "区别",
            "不同",
            "差异",
            "计算",
            "统计",
            "数量",
            "哪里",
            "在哪",
            "怎么查",
            "是否",
            "能不能",
            "可不可以",
            "期限",
            "时限",
            "时间",
            "费用",
            "多少钱",
            "成本",
        ];

        for verb in &complex_verbs {
            if input.contains(verb) {
                return Complexity::Complex;
            }
        }

        for verb in &medium_verbs {
            if input.contains(verb) {
                return Complexity::Medium;
            }
        }

        for verb in &simple_verbs {
            if input.contains(verb) {
                return Complexity::Simple;
            }
        }

        // 根据输入长度推断
        if input.chars().count() > 100 {
            Complexity::Complex
        } else if input.chars().count() > 30 {
            Complexity::Medium
        } else {
            Complexity::Simple
        }
    }
}

impl Default for ComplexityAssessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_writing() {
        let assessor = ComplexityAssessor::new();
        assert_eq!(assessor.assess("撰写专利申请文件"), Complexity::Complex);
    }

    #[test]
    fn test_medium_analysis() {
        let assessor = ComplexityAssessor::new();
        assert_eq!(assessor.assess("分析这个专利的新颖性"), Complexity::Medium);
    }

    #[test]
    fn test_simple_query() {
        let assessor = ComplexityAssessor::new();
        assert_eq!(assessor.assess("什么是三步法"), Complexity::Simple);
    }

    #[test]
    fn test_complex_long_input() {
        let assessor = ComplexityAssessor::new();
        let long_input = "我有一个关于人工智能图像识别的技术方案需要申请专利".repeat(5);
        assert_eq!(assessor.assess(&long_input), Complexity::Complex);
    }
}
