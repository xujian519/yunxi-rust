//! 专业领域检测器
//!
//! 基于关键词匹配检测用户输入的专业领域。

use crate::types::Domain;

/// 领域检测器
pub struct DomainDetector {
    patent_keywords: Vec<&'static str>,
    trademark_keywords: Vec<&'static str>,
    copyright_keywords: Vec<&'static str>,
    legal_keywords: Vec<&'static str>,
}

impl DomainDetector {
    pub fn new() -> Self {
        Self {
            patent_keywords: vec![
                // ---- 核心术语 ----
                "专利",
                "权利要求",
                "说明书",
                "三步法",
                "创造性",
                "新颖性",
                "实用性",
                "审查",
                "OA",
                "答复",
                "驳回",
                "授权",
                "发明",
                "实用新型",
                "外观设计",
                "独立权利要求",
                "从属权利要求",
                "技术方案",
                "实施例",
                "附图",
                "摘要",
                "申请文件",
                "交底书",
                "技术特征",
                "等同侵权",
                "保护范围",
                "审查指南",
                "IPC",
                "公开",
                "公告",
                "优先权",
                "宽限期",
                "充分公开",
                "清楚完整",
                // ---- 新颖性相关 ----
                "新颖性宽限期",
                "丧失新颖性",
                "对比文件",
                "现有技术",
                "抵触申请",
                "单独对比",
                "实质相同",
                "惯用手段替换",
                "出版物公开",
                "使用公开",
                "现有设计",
                "申请日",
                "优先权日",
                // ---- 创造性相关 ----
                "显而易见",
                "技术启示",
                "区别特征",
                "技术效果",
                "预料不到",
                "实际解决的技术问题",
                "结合启示",
                "公知常识",
                "辅助判断因素",
                "克服技术偏见",
                "商业成功",
                "解决技术难题",
                // ---- 审查/OA 相关 ----
                "审查意见",
                "第一次审查意见",
                "审查通知书",
                "意见陈述书",
                "补正",
                "修改超范围",
                "分案申请",
                "提前公开",
                "实质审查",
                "初步审查",
                "形式审查",
                "驳回决定",
                "复审请求",
                "复审委员会",
                "无效宣告",
                "无效请求",
                "口审",
                // ---- 撰写相关 ----
                "撰写",
                "权利要求书",
                "独立项",
                "从属项",
                "前序部分",
                "特征部分",
                "过渡词",
                "其特征在于",
                "包含",
                "由…组成",
                "开放式",
                "封闭式",
                "功能性限定",
                "方法权利要求",
                "产品权利要求",
                "用途权利要求",
                "多项从属",
                // ---- 侵权相关 ----
                "侵权",
                "专利侵权",
                "等同原则",
                "禁止反悔",
                "捐献原则",
                "全面覆盖原则",
                "帮助侵权",
                "间接侵权",
                "诱导侵权",
                "合法来源",
                "先用权",
                "Bolar例外",
                "科研例外",
                "临时过境",
                "权利用尽",
                "专利权效力",
                "举证责任",
                "举证责任倒置",
                "新产品制造方法",
                // ---- 诉讼/程序 ----
                "专利诉讼",
                "侵权诉讼",
                "侵权判定",
                "侵权抗辩",
                "权利要求解释",
                "内部证据",
                "外部证据",
                "判决",
                "上诉",
                "再审",
                "中止诉讼",
                "诉前禁令",
                "保全",
                "损害赔偿",
                "合理开支",
                // ---- 分类/检索 ----
                "国际专利分类",
                "联合专利分类",
                "CPC分类",
                "专利检索",
                "专利分析",
                "专利地图",
                "专利预警",
                "专利布局",
                "专利池",
                "标准必要专利",
                "FRAND",
                // ---- 其他 ----
                "职务发明",
                "共同发明",
                "发明人",
                "申请人",
                "专利权人",
                "许可",
                "转让",
                "质押",
                "强制许可",
                "PCT",
                "国际申请",
                "国家阶段",
                "进入中国国家阶段",
                "专利代理",
                "代理机构",
                "代理人",
            ],
            trademark_keywords: vec![
                "商标",
                "注册商标",
                "商标注册",
                "商标申请",
                "商标异议",
                "驰名商标",
                "近似商标",
                "商品分类",
                "商标权",
                "商标侵权",
                "商标续展",
                "商标转让",
                "商标许可",
                "商标无效",
                "商标驳回复审",
                "商标评审",
                "尼斯分类",
                "商品类似群",
                "服务商标",
                "集体商标",
                "证明商标",
                "地理标志",
                "商标使用证据",
                "连续三年不使用",
                "撤三",
                "商标监测",
                "商标检索",
                "商标查询",
                "显著特征",
                "识别性",
                "混淆可能性",
                "恶意注册",
                "抢注",
                "在先权利",
                "商标代理",
                "商标局",
                "商标评审委员会",
                "tm",
                "R标",
                "立体商标",
                "声音商标",
                "颜色组合",
                "防御商标",
                "联合商标",
                "商标异议申请",
                "商标无效宣告",
                "商标撤销",
                "马德里商标",
                "国际注册",
            ],
            copyright_keywords: vec![
                "版权",
                "著作权",
                "作品",
                "发表权",
                "署名权",
                "修改权",
                "保护作品完整权",
                "复制权",
                "发行权",
                "出租权",
                "展览权",
                "表演权",
                "放映权",
                "广播权",
                "信息网络传播权",
                "摄制权",
                "改编权",
                "翻译权",
                "汇编权",
                "著作人身权",
                "著作财产权",
                "软件著作权",
                "计算机软件",
                "源代码",
                "开源协议",
                "合理使用",
                "法定许可",
                "强制许可",
                "版权登记",
                "版权侵权",
                "抄袭",
                "剽窃",
                "独创性",
                "思想表达二分法",
                "CC协议",
                "gpl",
                "mit",
                "bsd",
                "ai生成",
                "数据集",
                "邻接权",
                "表演者权",
                "开源",
            ],
            legal_keywords: vec![
                // ---- 通用法律 ----
                "合同",
                "诉讼",
                "法律",
                "判决",
                "仲裁",
                "侵权",
                "赔偿",
                "法院",
                "法庭",
                "原告",
                "被告",
                "上诉",
                "再审",
                "申诉",
                "起诉",
                "立案",
                "受理",
                "审理",
                "执行",
                "强制执行",
                // ---- 合同/商业 ----
                "劳动合同",
                "买卖合同",
                "租赁合同",
                "借款合同",
                "担保",
                "抵押",
                "质押",
                "保证",
                "留置",
                "违约",
                "解除合同",
                "损害赔偿",
                "缔约过失",
                "公司法",
                "股东",
                "股权",
                "董事",
                "监事",
                "破产",
                "清算",
                "重组",
                // ---- 知识产权法 ----
                "知识产权",
                "专利法",
                "商标法",
                "著作权法",
                "反不正当竞争",
                "商业秘密",
                "技术秘密",
                "专利法实施细则",
                "审查指南",
                // ---- 程序法 ----
                "民事诉讼法",
                "行政诉讼法",
                "刑事诉讼法",
                "证据",
                "举证",
                "质证",
                "鉴定",
                "公证",
                "诉前保全",
                "证据保全",
                "财产保全",
                "管辖",
                "级别管辖",
                "地域管辖",
                "时效",
                "诉讼时效",
                "除斥期间",
                // ---- 其他 ----
                "行政法规",
                "司法解释",
                "最高法院",
                "法律意见",
                "合规",
                "尽职调查",
                "技术转让",
                "技术合同",
                "许可协议",
                "知识产权法院",
                "指导案例",
                "典型案例",
                "法律适用",
            ],
        }
    }

    /// 检测输入所属领域
    pub fn detect(&self, input: &str) -> (Domain, f64) {
        let input_lower = input.to_lowercase();

        let patent_score = self.score_domain(&input_lower, &self.patent_keywords);
        let trademark_score = self.score_domain(&input_lower, &self.trademark_keywords);
        let copyright_score = self.score_domain(&input_lower, &self.copyright_keywords);
        let legal_score = self.score_domain(&input_lower, &self.legal_keywords);

        let scores = [
            (Domain::Patent, patent_score),
            (Domain::Trademark, trademark_score),
            (Domain::Copyright, copyright_score),
            (Domain::Legal, legal_score),
        ];

        let (best_domain, best_score) = scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((Domain::General, 0.0));

        // 至少需要匹配一个关键词才认为是专业领域
        if best_score > 0.0 {
            (best_domain, best_score)
        } else {
            (Domain::General, 0.5)
        }
    }

    fn score_domain(&self, input: &str, keywords: &[&str]) -> f64 {
        let matches = keywords.iter().filter(|kw| input.contains(**kw)).count();
        if matches == 0 {
            return 0.0;
        }
        // 匹配越多分数越高，但归一化到 [0, 1]
        let base = matches as f64 / keywords.len().min(matches * 3) as f64;
        (base * 0.7 + 0.3).min(1.0)
    }
}

impl Default for DomainDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_patent() {
        let detector = DomainDetector::new();
        let (domain, score) = detector.detect("帮我分析这个专利的新颖性");
        assert_eq!(domain, Domain::Patent);
        assert!(score > 0.5);
    }

    #[test]
    fn test_detect_trademark() {
        let detector = DomainDetector::new();
        let (domain, _) = detector.detect("我想注册一个商标");
        assert_eq!(domain, Domain::Trademark);
    }

    #[test]
    fn test_detect_general() {
        let detector = DomainDetector::new();
        let (domain, _) = detector.detect("今天天气怎么样");
        assert_eq!(domain, Domain::General);
    }

    #[test]
    fn test_detect_patent_writing() {
        let detector = DomainDetector::new();
        let (domain, score) = detector.detect("撰写专利申请文件，包括权利要求书和说明书");
        assert_eq!(domain, Domain::Patent);
        assert!(score > 0.5);
    }

    #[test]
    fn test_detect_legal() {
        let detector = DomainDetector::new();
        let (domain, _) = detector.detect("这个合同纠纷应该怎么处理");
        assert_eq!(domain, Domain::Legal);
    }

    #[test]
    fn test_detect_trademark_tm() {
        let detector = DomainDetector::new();
        let (domain, _) = detector.detect("这个TM标志可以使用吗");
        assert_eq!(domain, Domain::Trademark);
    }

    #[test]
    fn test_detect_copyright_opensource() {
        let detector = DomainDetector::new();
        let (domain, _) = detector.detect("这个项目的GPL协议有什么限制");
        assert_eq!(domain, Domain::Copyright);
    }

    #[test]
    fn test_detect_legal_guidance() {
        let detector = DomainDetector::new();
        let (domain, _) = detector.detect("有没有相关的指导案例可以参考");
        assert_eq!(domain, Domain::Legal);
    }
}
