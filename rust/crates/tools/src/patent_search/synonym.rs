// 同义词词典数据结构 + SynonymSearch 工具

use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynonymSearchInput {
    pub operation: String,
    #[serde(default)]
    pub term: Option<String>,
    #[serde(default)]
    pub terms: Option<Vec<String>>,
    #[serde(default)]
    #[allow(dead_code)]
    pub domain: Option<String>, // 保留原因: 预留给按技术领域过滤同义词扩展
    #[serde(default)]
    pub accuracy: Option<String>,
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub exclusions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub(super) struct SynonymEntry {
    pub(super) chinese: &'static str,
    pub(super) english: &'static str,
    pub(super) domain: &'static str,
    pub(super) importance: f64,
    pub(super) synonyms: &'static [&'static str],
    pub(super) related_terms: &'static [&'static str],
}

// 同义词词典常量 (70+条目, 7个技术领域)
pub(super) const SYNONYM_DICT: &[SynonymEntry] = &[
    // AI领域 (10条)
    SynonymEntry {
        chinese: "机器学习",
        english: "machine learning",
        domain: "AI",
        importance: 1.0,
        synonyms: &["ML", "机器学习方法", "统计学习", "算法学习"],
        related_terms: &["深度学习", "监督学习", "无监督学习"],
    },
    SynonymEntry {
        chinese: "深度学习",
        english: "deep learning",
        domain: "AI",
        importance: 1.0,
        synonyms: &["DL", "深层学习", "神经网络学习", "深度神经网络"],
        related_terms: &["神经网络", "卷积神经网络", "循环神经网络"],
    },
    SynonymEntry {
        chinese: "神经网络",
        english: "neural network",
        domain: "AI",
        importance: 1.0,
        synonyms: &["NN", "人工神经网络", "ANN", "神经网"],
        related_terms: &["深度学习", "卷积神经网络", "感知器"],
    },
    SynonymEntry {
        chinese: "自然语言处理",
        english: "natural language processing",
        domain: "AI",
        importance: 0.9,
        synonyms: &["NLP", "自然语言理解", "文本处理", "语言理解"],
        related_terms: &["文本挖掘", "语音识别", "机器翻译"],
    },
    SynonymEntry {
        chinese: "计算机视觉",
        english: "computer vision",
        domain: "AI",
        importance: 0.9,
        synonyms: &["CV", "图像识别", "视觉处理", "机器视觉"],
        related_terms: &["图像处理", "目标检测", "图像分割"],
    },
    SynonymEntry {
        chinese: "强化学习",
        english: "reinforcement learning",
        domain: "AI",
        importance: 0.8,
        synonyms: &["RL", "增强学习", "再励学习"],
        related_terms: &["深度强化学习", "Q学习", "策略梯度"],
    },
    SynonymEntry {
        chinese: "知识图谱",
        english: "knowledge graph",
        domain: "AI",
        importance: 0.8,
        synonyms: &["KG", "知识库", "语义网络", "本体"],
        related_terms: &["图数据库", "实体链接", "关系抽取"],
    },
    SynonymEntry {
        chinese: "迁移学习",
        english: "transfer learning",
        domain: "AI",
        importance: 0.7,
        synonyms: &["领域自适应", "知识迁移"],
        related_terms: &["预训练模型", "微调"],
    },
    SynonymEntry {
        chinese: "生成对抗网络",
        english: "generative adversarial network",
        domain: "AI",
        importance: 0.7,
        synonyms: &["GAN", "对抗网络", "对抗生成网络"],
        related_terms: &["生成模型", "判别器", "生成器"],
    },
    SynonymEntry {
        chinese: "注意力机制",
        english: "attention mechanism",
        domain: "AI",
        importance: 0.7,
        synonyms: &["attention", "注意力", "自注意力"],
        related_terms: &["Transformer", "多头注意力"],
    },
    // 通信领域 (10条)
    SynonymEntry {
        chinese: "5G",
        english: "5th generation",
        domain: "Communication",
        importance: 1.0,
        synonyms: &["第五代移动通信", "5th generation mobile", "NR"],
        related_terms: &["LTE", "移动通信", "无线接入"],
    },
    SynonymEntry {
        chinese: "LTE",
        english: "long term evolution",
        domain: "Communication",
        importance: 0.9,
        synonyms: &["长期演进", "4G", "4G LTE"],
        related_terms: &["5G", "移动通信", "蜂窝网络"],
    },
    SynonymEntry {
        chinese: "物联网",
        english: "Internet of Things",
        domain: "Communication",
        importance: 0.9,
        synonyms: &["IoT", "物物互联", "传感网"],
        related_terms: &["传感器", "无线通信", "M2M"],
    },
    SynonymEntry {
        chinese: "无线通信",
        english: "wireless communication",
        domain: "Communication",
        importance: 0.8,
        synonyms: &["无线电", "无线传输", "无线链路"],
        related_terms: &["5G", "WiFi", "蓝牙"],
    },
    SynonymEntry {
        chinese: "调制解调",
        english: "modulation and demodulation",
        domain: "Communication",
        importance: 0.7,
        synonyms: &["modem", "调制", "解调"],
        related_terms: &["信号处理", "载波", "频移键控"],
    },
    SynonymEntry {
        chinese: "天线",
        english: "antenna",
        domain: "Communication",
        importance: 0.7,
        synonyms: &["天线阵列", "多天线", "智能天线"],
        related_terms: &["MIMO", "波束赋形"],
    },
    SynonymEntry {
        chinese: "基站",
        english: "base station",
        domain: "Communication",
        importance: 0.7,
        synonyms: &["NodeB", "eNodeB", "gNodeB", "接入点"],
        related_terms: &["移动通信", "无线接入", "蜂窝"],
    },
    SynonymEntry {
        chinese: "频谱",
        english: "spectrum",
        domain: "Communication",
        importance: 0.6,
        synonyms: &["频段", "频率", "带宽"],
        related_terms: &["频谱分配", "频谱效率"],
    },
    SynonymEntry {
        chinese: "编码",
        english: "coding",
        domain: "Communication",
        importance: 0.6,
        synonyms: &["信道编码", "纠错编码", "编码技术"],
        related_terms: &["解码", "误码率", "香农"],
    },
    SynonymEntry {
        chinese: "信道",
        english: "channel",
        domain: "Communication",
        importance: 0.6,
        synonyms: &["通信信道", "传输信道", "无线信道"],
        related_terms: &["信道估计", "信道编码"],
    },
    // 计算机领域 (10条)
    SynonymEntry {
        chinese: "操作系统",
        english: "operating system",
        domain: "Computer",
        importance: 0.9,
        synonyms: &["OS", "系统软件", "内核"],
        related_terms: &["进程管理", "内存管理", "文件系统"],
    },
    SynonymEntry {
        chinese: "数据库",
        english: "database",
        domain: "Computer",
        importance: 0.9,
        synonyms: &["DB", "数据存储", "数据库系统"],
        related_terms: &["SQL", "事务", "索引"],
    },
    SynonymEntry {
        chinese: "云计算",
        english: "cloud computing",
        domain: "Computer",
        importance: 0.8,
        synonyms: &["云", "云服务", "云端计算"],
        related_terms: &["虚拟化", "分布式系统", "容器"],
    },
    SynonymEntry {
        chinese: "区块链",
        english: "blockchain",
        domain: "Computer",
        importance: 0.8,
        synonyms: &["分布式账本", "链", "区块"],
        related_terms: &["智能合约", "共识机制", "加密货币"],
    },
    SynonymEntry {
        chinese: "容器",
        english: "container",
        domain: "Computer",
        importance: 0.7,
        synonyms: &["容器化", "Docker", "容器技术"],
        related_terms: &["微服务", "Kubernetes", "虚拟化"],
    },
    SynonymEntry {
        chinese: "微服务",
        english: "microservices",
        domain: "Computer",
        importance: 0.7,
        synonyms: &["微服务架构", "微服务化"],
        related_terms: &["容器", "服务网格", "API网关"],
    },
    SynonymEntry {
        chinese: "负载均衡",
        english: "load balancing",
        domain: "Computer",
        importance: 0.6,
        synonyms: &["负载", "流量分发"],
        related_terms: &["反向代理", "高可用", "分布式"],
    },
    SynonymEntry {
        chinese: "缓存",
        english: "cache",
        domain: "Computer",
        importance: 0.6,
        synonyms: &["缓冲", "高速缓存"],
        related_terms: &["Redis", "Memcached", "CDN"],
    },
    SynonymEntry {
        chinese: "虚拟化",
        english: "virtualization",
        domain: "Computer",
        importance: 0.6,
        synonyms: &["虚拟机", "虚拟", "VM"],
        related_terms: &["Hypervisor", "容器", "云"],
    },
    SynonymEntry {
        chinese: "中间件",
        english: "middleware",
        domain: "Computer",
        importance: 0.6,
        synonyms: &["中间件系统", "消息队列"],
        related_terms: &["消息中间件", "应用服务器"],
    },
    // 材料领域 (10条)
    SynonymEntry {
        chinese: "纳米材料",
        english: "nanomaterial",
        domain: "Materials",
        importance: 1.0,
        synonyms: &["纳米", "纳米级材料"],
        related_terms: &["石墨烯", "纳米颗粒"],
    },
    SynonymEntry {
        chinese: "复合材料",
        english: "composite material",
        domain: "Materials",
        importance: 0.9,
        synonyms: &["复合", "复合物"],
        related_terms: &["基体", "增强体"],
    },
    SynonymEntry {
        chinese: "高分子",
        english: "polymer",
        domain: "Materials",
        importance: 0.9,
        synonyms: &["聚合物", "大分子", "高聚物"],
        related_terms: &["塑料", "橡胶", "纤维"],
    },
    SynonymEntry {
        chinese: "陶瓷",
        english: "ceramic",
        domain: "Materials",
        importance: 0.7,
        synonyms: &["陶瓷材料", "结构陶瓷"],
        related_terms: &["氧化物", "烧结"],
    },
    SynonymEntry {
        chinese: "合金",
        english: "alloy",
        domain: "Materials",
        importance: 0.7,
        synonyms: &["合金材料", "金属合金"],
        related_terms: &["金属", "固溶体"],
    },
    SynonymEntry {
        chinese: "涂层",
        english: "coating",
        domain: "Materials",
        importance: 0.6,
        synonyms: &["镀层", "薄膜", "表面涂层"],
        related_terms: &["表面处理", "沉积"],
    },
    SynonymEntry {
        chinese: "催化剂",
        english: "catalyst",
        domain: "Materials",
        importance: 0.8,
        synonyms: &["催化", "催化材料"],
        related_terms: &["催化反应", "活性位点"],
    },
    SynonymEntry {
        chinese: "聚合物",
        english: "polymer",
        domain: "Materials",
        importance: 0.8,
        synonyms: &["高聚物", "聚合体"],
        related_terms: &["聚合反应", "单体"],
    },
    SynonymEntry {
        chinese: "石墨烯",
        english: "graphene",
        domain: "Materials",
        importance: 0.8,
        synonyms: &["石墨", "单层石墨"],
        related_terms: &["碳材料", "纳米材料"],
    },
    SynonymEntry {
        chinese: "晶体",
        english: "crystal",
        domain: "Materials",
        importance: 0.7,
        synonyms: &["结晶", "晶态"],
        related_terms: &["晶格", "单晶", "多晶"],
    },
    // 医疗领域 (10条)
    SynonymEntry {
        chinese: "医学影像",
        english: "medical imaging",
        domain: "Medical",
        importance: 0.9,
        synonyms: &["医学图像", "影像诊断"],
        related_terms: &["CT", "MRI", "超声"],
    },
    SynonymEntry {
        chinese: "基因编辑",
        english: "gene editing",
        domain: "Medical",
        importance: 1.0,
        synonyms: &["基因组编辑", "基因修饰"],
        related_terms: &["CRISPR", "基因治疗"],
    },
    SynonymEntry {
        chinese: "药物递送",
        english: "drug delivery",
        domain: "Medical",
        importance: 0.8,
        synonyms: &["给药", "药物输送", "靶向给药"],
        related_terms: &["纳米药物", "控释"],
    },
    SynonymEntry {
        chinese: "生物传感器",
        english: "biosensor",
        domain: "Medical",
        importance: 0.8,
        synonyms: &["生物传感", "传感器"],
        related_terms: &["检测", "诊断"],
    },
    SynonymEntry {
        chinese: "手术机器人",
        english: "surgical robot",
        domain: "Medical",
        importance: 0.8,
        synonyms: &["机器人手术", "微创手术机器人"],
        related_terms: &["达芬奇", "微创手术"],
    },
    SynonymEntry {
        chinese: "诊断",
        english: "diagnosis",
        domain: "Medical",
        importance: 0.7,
        synonyms: &["诊断技术", "临床诊断"],
        related_terms: &["筛查", "检测"],
    },
    SynonymEntry {
        chinese: "免疫治疗",
        english: "immunotherapy",
        domain: "Medical",
        importance: 0.8,
        synonyms: &["免疫", "免疫疗法"],
        related_terms: &["抗体", "癌症免疫治疗"],
    },
    SynonymEntry {
        chinese: "组织工程",
        english: "tissue engineering",
        domain: "Medical",
        importance: 0.7,
        synonyms: &["组织再生", "组织修复"],
        related_terms: &["干细胞", "支架"],
    },
    SynonymEntry {
        chinese: "体外诊断",
        english: "in vitro diagnosis",
        domain: "Medical",
        importance: 0.7,
        synonyms: &["IVD", "体外检测"],
        related_terms: &["试剂盒", "POCT"],
    },
    SynonymEntry {
        chinese: "内窥镜",
        english: "endoscope",
        domain: "Medical",
        importance: 0.7,
        synonyms: &["内镜", "内窥"],
        related_terms: &["胃镜", "肠镜", "微创"],
    },
    // 电子领域 (10条)
    SynonymEntry {
        chinese: "半导体",
        english: "semiconductor",
        domain: "Electronics",
        importance: 1.0,
        synonyms: &["半导体器件", "半导体材料"],
        related_terms: &["硅", "芯片", "晶体管"],
    },
    SynonymEntry {
        chinese: "芯片",
        english: "chip",
        domain: "Electronics",
        importance: 1.0,
        synonyms: &["集成电路", "IC", "微芯片"],
        related_terms: &["处理器", "半导体"],
    },
    SynonymEntry {
        chinese: "显示屏",
        english: "display",
        domain: "Electronics",
        importance: 0.8,
        synonyms: &["显示器", "显示面板", "屏幕"],
        related_terms: &["LCD", "OLED", "触摸屏"],
    },
    SynonymEntry {
        chinese: "传感器",
        english: "sensor",
        domain: "Electronics",
        importance: 0.8,
        synonyms: &["传感", "感应器"],
        related_terms: &["智能传感器", "物联网"],
    },
    SynonymEntry {
        chinese: "电池",
        english: "battery",
        domain: "Electronics",
        importance: 0.8,
        synonyms: &["电池技术", "蓄电池", "锂电池"],
        related_terms: &["储能", "正极", "负极"],
    },
    SynonymEntry {
        chinese: "存储器",
        english: "memory",
        domain: "Electronics",
        importance: 0.7,
        synonyms: &["内存", "存储", "存储芯片"],
        related_terms: &["DRAM", "闪存", "SSD"],
    },
    SynonymEntry {
        chinese: "光电",
        english: "optoelectronic",
        domain: "Electronics",
        importance: 0.7,
        synonyms: &["光电子", "光电技术"],
        related_terms: &["激光", "光通信", "LED"],
    },
    SynonymEntry {
        chinese: "集成电路",
        english: "integrated circuit",
        domain: "Electronics",
        importance: 0.8,
        synonyms: &["IC", "芯片", "集成芯片"],
        related_terms: &["半导体", "处理器"],
    },
    SynonymEntry {
        chinese: "信号处理",
        english: "signal processing",
        domain: "Electronics",
        importance: 0.7,
        synonyms: &["数字信号处理", "DSP"],
        related_terms: &["滤波", "傅里叶变换"],
    },
    SynonymEntry {
        chinese: "电源管理",
        english: "power management",
        domain: "Electronics",
        importance: 0.6,
        synonyms: &["PMIC", "电源"],
        related_terms: &["电压调节", "功耗"],
    },
    // 化工领域 (10条)
    SynonymEntry {
        chinese: "催化反应",
        english: "catalytic reaction",
        domain: "Chemical",
        importance: 0.9,
        synonyms: &["催化", "催化转化"],
        related_terms: &["催化剂", "反应器"],
    },
    SynonymEntry {
        chinese: "分离纯化",
        english: "separation and purification",
        domain: "Chemical",
        importance: 0.8,
        synonyms: &["分离", "纯化", "提纯"],
        related_terms: &["蒸馏", "萃取", "色谱"],
    },
    SynonymEntry {
        chinese: "高分子合成",
        english: "polymer synthesis",
        domain: "Chemical",
        importance: 0.8,
        synonyms: &["聚合", "聚合合成"],
        related_terms: &["聚合反应", "单体"],
    },
    SynonymEntry {
        chinese: "表面处理",
        english: "surface treatment",
        domain: "Chemical",
        importance: 0.7,
        synonyms: &["表面改性", "表面工程"],
        related_terms: &["涂层", "蚀刻"],
    },
    SynonymEntry {
        chinese: "电化学",
        english: "electrochemistry",
        domain: "Chemical",
        importance: 0.7,
        synonyms: &["电化学方法"],
        related_terms: &["电解", "电池", "腐蚀"],
    },
    SynonymEntry {
        chinese: "发酵",
        english: "fermentation",
        domain: "Chemical",
        importance: 0.7,
        synonyms: &["发酵工程", "发酵技术"],
        related_terms: &["微生物", "生物反应器"],
    },
    SynonymEntry {
        chinese: "萃取",
        english: "extraction",
        domain: "Chemical",
        importance: 0.6,
        synonyms: &["提取", "萃取分离"],
        related_terms: &["溶剂萃取", "超临界萃取"],
    },
    SynonymEntry {
        chinese: "聚合反应",
        english: "polymerization",
        domain: "Chemical",
        importance: 0.7,
        synonyms: &["聚合", "加聚", "缩聚"],
        related_terms: &["聚合物", "高分子"],
    },
    SynonymEntry {
        chinese: "氧化还原",
        english: "redox",
        domain: "Chemical",
        importance: 0.6,
        synonyms: &["氧化", "还原"],
        related_terms: &["氧化剂", "还原剂"],
    },
    SynonymEntry {
        chinese: "合成方法",
        english: "synthesis method",
        domain: "Chemical",
        importance: 0.6,
        synonyms: &["合成", "制备方法"],
        related_terms: &["化学合成", "有机合成"],
    },
];

// ============================================================================
// 工具1: SynonymSearch - 专利同义词词典
// ============================================================================

pub fn synonym_search(input: SynonymSearchInput) -> Result<Value, String> {
    match input.operation.as_str() {
        "lookup" => {
            let term = input
                .term
                .ok_or("lookup operation requires 'term' parameter")?;
            if term.len() < 2 {
                return Err("term must be at least 2 characters".to_string());
            }
            lookup_synonym(&term)
        }
        "expand" => {
            let terms = input
                .terms
                .ok_or("expand operation requires 'terms' parameter")?;
            expand_synonyms(&terms)
        }
        "detect_domain" => {
            let term = input
                .term
                .ok_or("detect_domain operation requires 'term' parameter")?;
            detect_domain(&term)
        }
        "build_query" => {
            let terms = input
                .terms
                .ok_or("build_query operation requires 'terms' parameter")?;
            let accuracy = input.accuracy.unwrap_or_else(|| "Medium".to_string());
            let field = input.field.unwrap_or_else(|| "all".to_string());
            let exclusions = input.exclusions.unwrap_or_default();
            build_search_query(&terms, &accuracy, &field, &exclusions)
        }
        "build_progressive" => {
            let terms = input
                .terms
                .ok_or("build_progressive operation requires 'terms' parameter")?;
            let field = input.field.unwrap_or_else(|| "all".to_string());
            build_progressive_query(&terms, &field)
        }
        "stats" => get_dictionary_stats(),
        _ => Err(format!("unknown operation: {}", input.operation)),
    }
}

#[allow(clippy::unnecessary_wraps)]
fn lookup_synonym(term: &str) -> Result<Value, String> {
    let term_lower = term.to_lowercase();

    // 精确匹配
    for entry in SYNONYM_DICT {
        if entry.chinese == term || entry.english.to_lowercase() == term_lower {
            return Ok(json!({
                "found": true,
                "match_type": "exact",
                "entry": {
                    "chinese": entry.chinese,
                    "english": entry.english,
                    "domain": entry.domain,
                    "importance": entry.importance,
                    "synonyms": entry.synonyms,
                    "related_terms": entry.related_terms
                }
            }));
        }
    }

    // 子串匹配
    for entry in SYNONYM_DICT {
        if entry.chinese.contains(term) || entry.english.to_lowercase().contains(&term_lower) {
            return Ok(json!({
                "found": true,
                "match_type": "substring",
                "entry": {
                    "chinese": entry.chinese,
                    "english": entry.english,
                    "domain": entry.domain,
                    "importance": entry.importance,
                    "synonyms": entry.synonyms,
                    "related_terms": entry.related_terms
                }
            }));
        }
    }

    // 同义词匹配
    for entry in SYNONYM_DICT {
        for syn in entry.synonyms {
            if syn.to_lowercase().contains(&term_lower) {
                return Ok(json!({
                    "found": true,
                    "match_type": "synonym",
                    "entry": {
                        "chinese": entry.chinese,
                        "english": entry.english,
                        "domain": entry.domain,
                        "importance": entry.importance,
                        "synonyms": entry.synonyms,
                        "related_terms": entry.related_terms
                    }
                }));
            }
        }
    }

    Ok(json!({
        "found": false,
        "message": format!("Term '{}' not found in dictionary", term)
    }))
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn expand_synonyms(terms: &[String]) -> Result<Value, String> {
    let mut all_synonyms = Vec::new();

    for term in terms {
        if let Ok(result) = lookup_synonym(term) {
            if result["found"].as_bool().unwrap_or(false) {
                let entry = &result["entry"];
                if let Some(cn) = entry["chinese"].as_str() {
                    all_synonyms.push(cn.to_string());
                }
                if let Some(en) = entry["english"].as_str() {
                    all_synonyms.push(en.to_string());
                }
                if let Some(syns) = entry["synonyms"].as_array() {
                    for syn in syns {
                        if let Some(s) = syn.as_str() {
                            all_synonyms.push(s.to_string());
                        }
                    }
                }
            } else {
                all_synonyms.push(term.clone());
            }
        } else {
            all_synonyms.push(term.clone());
        }
    }

    // 去重
    all_synonyms.sort();
    all_synonyms.dedup();

    Ok(json!({
        "input_terms": terms,
        "expanded_synonyms": all_synonyms,
        "total_count": all_synonyms.len()
    }))
}

#[allow(clippy::unnecessary_wraps)]
fn detect_domain(text: &str) -> Result<Value, String> {
    let mut domain_scores = std::collections::HashMap::new();

    for entry in SYNONYM_DICT {
        let mut count = 0;

        // 检查中文术语
        if text.contains(entry.chinese) {
            count += 1;
        }

        // 检查英文术语
        if text.to_lowercase().contains(&entry.english.to_lowercase()) {
            count += 1;
        }

        // 检查同义词
        for syn in entry.synonyms {
            if text.to_lowercase().contains(&syn.to_lowercase()) {
                count += 1;
            }
        }

        if count > 0 {
            *domain_scores.entry(entry.domain).or_insert(0.0) +=
                f64::from(count) * entry.importance;
        }
    }

    let mut domains: Vec<_> = domain_scores.into_iter().collect();
    domains.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(json!({
        "text": text,
        "detected_domains": domains.iter().map(|(d, s)| json!({
            "domain": d,
            "score": s
        })).collect::<Vec<_>>(),
        "primary_domain": domains.first().map(|(d, _)| *d)
    }))
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn build_search_query(
    terms: &[String],
    accuracy: &str,
    field: &str,
    exclusions: &[String],
) -> Result<Value, String> {
    let field_prefix = match field {
        "intitle" => "intitle:",
        "inabstract" => "inabstract:",
        "inclaims" => "inclaims:",
        _ => "",
    };

    let mut expanded_terms = Vec::new();
    for term in terms {
        if let Ok(result) = lookup_synonym(term) {
            if result["found"].as_bool().unwrap_or(false) {
                let entry = &result["entry"];
                expanded_terms.push(entry["chinese"].as_str().unwrap_or_default().to_string());
                expanded_terms.push(entry["english"].as_str().unwrap_or_default().to_string());

                // 根据精度限制同义词数量
                let syn_limit = match accuracy {
                    "High" => 0,   // 只用主术语
                    "Medium" => 2, // 用主术语+2个同义词
                    _ => 5,        // Low精度用更多同义词
                };

                if let Some(syns) = entry["synonyms"].as_array() {
                    for (i, syn) in syns.iter().enumerate() {
                        if i < syn_limit {
                            expanded_terms.push(syn.as_str().unwrap_or_default().to_string());
                        }
                    }
                }
            } else {
                expanded_terms.push(term.clone());
            }
        } else {
            expanded_terms.push(term.clone());
        }
    }

    // 去重
    expanded_terms.sort();
    expanded_terms.dedup();

    // 构建查询
    let operator = match accuracy {
        "High" => " AND ",
        _ => " OR ",
    };

    let mut query_parts: Vec<String> = expanded_terms
        .iter()
        .map(|t| format!("{field_prefix}{t}"))
        .collect();

    // 添加排除词
    for exclusion in exclusions {
        query_parts.push(format!("-{exclusion}"));
    }

    let query = if query_parts.is_empty() {
        "*".to_string()
    } else {
        query_parts.join(operator)
    };

    Ok(json!({
        "input_terms": terms,
        "accuracy_level": accuracy,
        "field": field,
        "expanded_terms": expanded_terms,
        "query": query,
        "exclusions": exclusions
    }))
}

fn build_progressive_query(terms: &[String], field: &str) -> Result<Value, String> {
    let stage1 = build_search_query(terms, "High", field, &[])?;
    let stage2 = build_search_query(terms, "Medium", field, &[])?;
    let stage3 = build_search_query(terms, "Low", field, &[])?;

    Ok(json!({
        "input_terms": terms,
        "field": field,
        "stages": [
            {
                "stage": 1,
                "name": "高精度检索",
                "strategy": "仅用主术语，AND连接",
                "query": stage1["query"]
            },
            {
                "stage": 2,
                "name": "中等精度检索",
                "strategy": "主术语+部分同义词，混合AND/OR",
                "query": stage2["query"]
            },
            {
                "stage": 3,
                "name": "低精度扩展检索",
                "strategy": "包含更多同义词，OR连接",
                "query": stage3["query"]
            }
        ]
    }))
}

#[allow(clippy::unnecessary_wraps)]
fn get_dictionary_stats() -> Result<Value, String> {
    let mut domain_counts = std::collections::HashMap::new();

    for entry in SYNONYM_DICT {
        *domain_counts.entry(entry.domain).or_insert(0) += 1;
    }

    Ok(json!({
        "total_terms": SYNONYM_DICT.len(),
        "domains": domain_counts,
        "domains_count": domain_counts.len()
    }))
}
