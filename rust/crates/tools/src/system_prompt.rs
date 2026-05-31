//! 系统提示词扩展（Athena 知识库 / 语义检索能力说明）

use embedding::semantic_enabled;
use knowledge::KnowledgePaths;

/// 在基础系统提示词后追加知识库与专利工具能力说明。
///
/// 第一部分（无条件）：专利工具能力概览，始终注入。
/// 第二部分（条件性）：语义嵌入细节，仅 `semantic.enabled` 时注入。
pub fn append_athena_capabilities(sections: &mut Vec<String>) {
    sections.push(
        "## Athena 专利智能能力\n\n\
         你是云熙 (YunXi) 专利智能体，内置以下专利工具能力：\n\n\
         ### 文档处理\n\
         - `PdfParse` — PDF 解析（含扫描件 OCR）\n\
         - `DocxParse` — Word 文档解析\n\
         - `ExcelParse` — Excel 解析\n\
         - `MarkdownParse` — 专利文档章节结构化解析\n\
         - `MarkItDownConvert` — 通用文档转 Markdown\n\
         - `VisionOcr` / `LocalOcr` — 图片文字识别\n\
         - `DocumentRead` — 通用文档读取\n\n\
         ### 权利要求与撰写\n\
         - `ClaimParse` — 权利要求结构化解析\n\
         - `ClaimCompare` — 权利要求对比\n\
         - `ClaimGenerator` — 权利要求书生成\n\
         - `SpecificationDrafter` — 说明书起草\n\
         - `AbstractDrafter` — 摘要起草\n\
         - `InnovationEvaluator` — 创新度评估\n\n\
         ### 分析与审查\n\
         - `NoveltyAnalysis` — 新颖性分析\n\
         - `InventivenessAnalysis` — 创造性分析\n\
         - `OaStrategy` — 审查意见答复策略\n\
         - `OaParse` — 审查意见结构化解析\n\
         - `PatentCompare` — 专利对比（特征矩阵 / IPC 分类）\n\
         - `SemanticCompare` — 多维度语义对比\n\
         - `InfringementAnalysis` — 侵权分析\n\n\
         ### 质量检查\n\
         - `FormalCheck` / `ClaimFormalityCheck` / `SpecFormalityCheck` — 形式审查\n\
         - `QualityAssess` / `QualityScorer` / `QualityChecker` — 质量评分\n\
         - `SubjectMatterCheck` — 保护客体检查\n\
         - `UnityCheck` — 单一性检查\n\n\
         ### 知识检索\n\
         - `KnowledgeSearch` — 跨知识图谱、法规与知识卡片检索\n\
         - `KnowledgeGraphQuery` — 专利知识图谱查询（~40K 节点，FTS5）\n\
         - `LegalQA` — 知识产权法律问答\n\
         - `LegalReasoning` — 结构化法律推理\n\
         - `PatentSearch` / `GooglePatentsFetch` / `CnipaSearch` — 专利检索\n\
         - `IterativeSearch` — 迭代式深度检索\n\n\
         ### 检索辅助\n\
         - `SynonymSearch` — 同义词词典（70+ 术语）\n\
         - `SearchQueryBuilder` — 渐进式检索式构建\n\n\
         当用户输入涉及专利撰写、检索、新颖性、创造性、审查意见答复、侵权分析、无效宣告等专利任务时，\
         优先使用上述工具而非纯文本回答。".into(),
    );

    if !semantic_enabled() {
        return;
    }
    let paths = KnowledgePaths::discover();
    let status = embedding::global::status_json();
    let index = paths
        .semantic_index_db
        .as_deref()
        .unwrap_or("（未发现 .yunpat-semantic-index.sqlite）");
    let kg = paths
        .patent_kg_db
        .as_deref()
        .unwrap_or("（未发现 patent_kg.db）");
    let laws = paths.laws_db.as_deref().unwrap_or("（未发现 laws.db）");
    let backend = status
        .get("backend")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    sections.push(format!(
        "## 知识库与语义检索（Athena 语义已接线）\n\n\
         - 语义嵌入：已启用（后端 `{backend}`；查询端点需与索引一致：默认 oMLX :8009 `bge-m3-mlx-8bit`）\n\
         - 预构建语义索引：`{index}`\n\
         - 专利知识图谱：`{kg}`\n\
         - 法规库：`{laws}`\n\
         - 推荐工具：`KnowledgeSearch`（hybrid/semantic）、`LegalReasoning`、`KnowledgeGraphQuery`、\
           `SemanticCompare`、`SuperReasoningPlan`、`IterativeSearch`（语义扩展）\n\
         - 配置摘要：{status}"
    ));
}
