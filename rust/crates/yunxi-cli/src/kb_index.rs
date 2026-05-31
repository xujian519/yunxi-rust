//! 知识库向量索引 CLI：`yunxi kb index`

use embedding::global::shared_required;
use knowledge::search::UnifiedSearch;
use knowledge::KnowledgePaths;
use serde_json::json;

#[derive(Debug, Default)]
pub struct KbIndexOptions {
    pub index_kg: bool,
    pub index_laws: bool,
    pub batch_size: usize,
}

pub fn run_kb_index(opts: KbIndexOptions) -> Result<(), Box<dyn std::error::Error>> {
    if !opts.index_kg && !opts.index_laws {
        return Err("请至少指定 --kg 或 --laws".into());
    }

    if let Some(idx) = knowledge::PrebuiltSemanticIndex::open_default() {
        eprintln!(
            "提示: 已存在预构建语义库 {}（{} chunks，模型 {:?}），Markdown 知识无需 yunxi kb index。",
            idx.db_path().display(),
            idx.chunk_count(),
            idx.embedding_model()
        );
        eprintln!("本命令仅用于可选的 KG/法规辅助向量（~/.yunxi/vectors/vectors.db）。");
    }

    let _ = shared_required().map_err(|e| format!("{e}"))?;

    let paths = KnowledgePaths::discover();
    let engine = UnifiedSearch::new(
        paths.patent_kg_db.as_deref(),
        paths.laws_db.as_deref(),
        paths.card_index.as_deref(),
    );

    let batch = opts.batch_size.max(8);

    if opts.index_kg {
        println!("正在索引知识图谱向量（batch={batch}）…");
        let stats = engine
            .index_knowledge_graph(batch)
            .map_err(|e| format!("KG 索引失败: {e}"))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "collection": "knowledge_graph",
                "stats": stats,
            }))?
        );
    }

    if opts.index_laws {
        println!("正在索引法律法规向量（batch={batch}）…");
        let stats = engine
            .index_law_database(batch)
            .map_err(|e| format!("法规索引失败: {e}"))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "collection": "laws",
                "stats": stats,
            }))?
        );
    }

    println!(
        "\n嵌入服务状态: {}",
        serde_json::to_string_pretty(&embedding::status_json())?
    );
    Ok(())
}

pub fn parse_kb_subcommand(args: &[String]) -> Result<Option<KbIndexOptions>, String> {
    if args.first().map(String::as_str) != Some("kb") {
        return Ok(None);
    }
    let rest = &args[1..];
    if rest.first().map(String::as_str) != Some("index") {
        return Err("用法: yunxi kb index [--kg] [--laws] [--batch-size N]".into());
    }

    let mut opts = KbIndexOptions {
        index_kg: false,
        index_laws: false,
        batch_size: 32,
    };

    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--kg" => opts.index_kg = true,
            "--laws" => opts.index_laws = true,
            "--batch-size" => {
                i += 1;
                let v = rest
                    .get(i)
                    .ok_or_else(|| "缺少 --batch-size 参数值".to_string())?;
                opts.batch_size = v.parse().map_err(|_| format!("无效的 batch-size: {v}"))?;
            }
            flag if flag.starts_with("--batch-size=") => {
                opts.batch_size = flag[13..]
                    .parse()
                    .map_err(|_| format!("无效的 batch-size: {}", &flag[13..]))?;
            }
            other => return Err(format!("未知参数: {other}")),
        }
        i += 1;
    }

    if !opts.index_kg && !opts.index_laws {
        opts.index_kg = true;
        opts.index_laws = true;
    }

    Ok(Some(opts))
}
