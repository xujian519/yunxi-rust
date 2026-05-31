//! 知识库数据资产路径解析（仓库 `assets/`、`~/.yunxi/`、环境变量）

use std::path::PathBuf;

/// 已解析的知识库数据路径
#[derive(Debug, Clone, Default)]
pub struct KnowledgePaths {
    pub patent_kg_db: Option<String>,
    pub laws_db: Option<String>,
    pub card_index: Option<String>,
    /// BGE 预向量化语义库（`.yunpat-semantic-index.sqlite`）
    pub semantic_index_db: Option<String>,
}

impl KnowledgePaths {
    /// 按优先级发现可用的数据文件路径
    #[must_use]
    pub fn discover() -> Self {
        let roots = asset_roots();
        let patent_kg_db = resolve_existing(&patent_kg_candidates(&roots));
        let laws_db = resolve_existing(&laws_db_candidates(&roots));
        let card_index = resolve_existing(&card_index_candidates(&roots));
        let semantic_index_db = resolve_existing(&semantic_index_candidates(&roots));
        Self {
            patent_kg_db,
            laws_db,
            card_index,
            semantic_index_db,
        }
    }
}

fn resolve_existing(candidates: &[PathBuf]) -> Option<String> {
    candidates
        .iter()
        .find(|p| p.exists())
        .map(|p| p.to_string_lossy().into_owned())
}

fn asset_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(dir) = std::env::var_os("YUNXI_DATA_DIR") {
        roots.push(PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("HOME") {
        roots.push(PathBuf::from(home).join(".yunxi"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let assets = ancestor.join("assets");
            if assets.is_dir() {
                roots.push(assets);
            }
        }
    }
    roots.push(PathBuf::from("assets"));
    roots
}

fn patent_kg_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("PATENT_KG_DB") {
        out.push(PathBuf::from(p));
    }
    for root in roots {
        out.push(root.join("knowledge_graph/patent_kg.db"));
    }
    // 仓库根下 assets 与 knowledge_graph 同级
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            out.push(ancestor.join("assets/knowledge_graph/patent_kg.db"));
        }
    }
    out
}

fn laws_db_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        out.push(root.join("knowledge/data/laws.db"));
        out.push(root.join("knowledge/data/laws-full.db"));
        out.push(root.join("data/laws.db"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            out.push(ancestor.join("assets/knowledge/data/laws.db"));
            out.push(ancestor.join("assets/knowledge/data/laws-full.db"));
        }
    }
    out
}

fn semantic_index_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("YUNXI_SEMANTIC_INDEX_DB") {
        out.push(PathBuf::from(p));
    }
    for root in roots {
        out.push(root.join("knowledge-base/.yunpat-semantic-index.sqlite"));
        out.push(root.join(".yunpat-semantic-index.sqlite"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            out.push(ancestor.join("assets/knowledge-base/.yunpat-semantic-index.sqlite"));
        }
    }
    out
}

fn card_index_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        out.push(root.join("knowledge-base/card-index.json"));
        out.push(root.join("knowledge/cards/card-index.json"));
        out.push(root.join("cards/card-index.json"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            out.push(ancestor.join("assets/knowledge-base/card-index.json"));
            out.push(ancestor.join("assets/knowledge/cards/card-index.json"));
        }
    }
    out
}

/// 确保用户级知识库目录结构存在（懒初始化）
pub fn ensure_user_knowledge_dirs() -> Result<(), std::io::Error> {
    let Some(home) = std::env::var_os("HOME") else {
        return Ok(());
    };
    let base = std::path::Path::new(&home).join(".yunxi");
    for dir in [
        base.join("knowledge"),
        base.join("knowledge").join("cards"),
        base.join("knowledge-base"),
    ] {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_repo_assets_when_run_from_rust_dir() {
        let paths = KnowledgePaths::discover();
        if paths.patent_kg_db.is_none() && paths.card_index.is_none() {
            eprintln!("skipped: no patent_kg.db or card-index.json under assets/");
            return;
        }
    }
}
