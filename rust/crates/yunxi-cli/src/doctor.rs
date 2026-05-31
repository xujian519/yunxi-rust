//! 本机环境健康检查（`yunxi doctor`）

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use knowledge::KnowledgePaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

struct CheckLine {
    name: &'static str,
    status: CheckStatus,
    detail: String,
}

pub fn run_doctor() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = locate_repo_root();
    let mut lines = Vec::new();

    lines.push(check_llm_key());
    lines.push(check_config_placement(repo_root.as_deref()));
    lines.extend(check_knowledge_assets(repo_root.as_deref()));
    lines.push(check_patent_kg_canonical(repo_root.as_deref()));
    lines.push(check_omlx());
    lines.extend(check_patent_tools());
    lines.push(check_rust_tests_hint());

    println!("云熙智能体 — 环境检查 (yunxi doctor)\n");
    let mut failures = 0usize;
    let mut warnings = 0usize;
    for line in &lines {
        let icon = match line.status {
            CheckStatus::Ok => "✓",
            CheckStatus::Warn => "!",
            CheckStatus::Fail => "✗",
        };
        println!("{icon} {} — {}", line.name, line.detail);
        match line.status {
            CheckStatus::Fail => failures += 1,
            CheckStatus::Warn => warnings += 1,
            CheckStatus::Ok => {}
        }
    }
    println!();
    if failures > 0 {
        println!("结果: {failures} 项未通过，{warnings} 项警告。请先修复失败项再使用完整功能。");
        std::process::exit(1);
    }
    if warnings > 0 {
        println!("结果: 可用，但有 {warnings} 项警告（部分能力可能降级）。");
    } else {
        println!("结果: 本机环境检查通过。");
    }
    Ok(())
}

fn check_llm_key() -> CheckLine {
    let deepseek = env::var("DEEPSEEK_API_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty());
    let anthropic = env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            env::var("ANTHROPIC_AUTH_TOKEN")
                .ok()
                .filter(|v| !v.trim().is_empty())
        });
    if deepseek.is_some() {
        CheckLine {
            name: "LLM API",
            status: CheckStatus::Ok,
            detail: "DEEPSEEK_API_KEY 已设置（默认模型 deepseek-v4-pro）".to_string(),
        }
    } else if anthropic.is_some() {
        CheckLine {
            name: "LLM API",
            status: CheckStatus::Ok,
            detail: "Anthropic 凭据已设置".to_string(),
        }
    } else {
        CheckLine {
            name: "LLM API",
            status: CheckStatus::Fail,
            detail: "未设置 DEEPSEEK_API_KEY 或 ANTHROPIC_API_KEY；对话与 prompt 不可用"
                .to_string(),
        }
    }
}

fn check_config_placement(repo_root: Option<&Path>) -> CheckLine {
    let home = env::var("HOME").ok().map(PathBuf::from);
    let user_local = home.as_ref().map(|h| h.join(".yunxi/settings.local.json"));
    let user_exists = user_local.as_ref().is_some_and(|p| p.is_file());

    if let Some(root) = repo_root {
        let project_local = root.join(".yunxi/settings.local.json");
        if project_local.is_file() {
            let raw = std::fs::read_to_string(&project_local).unwrap_or_default();
            let has_inline_key = raw.contains("\"apiKey\"")
                && !raw.contains("\"apiKey\": \"\"")
                && !raw.contains("\"apiKey\":\"\"");
            if has_inline_key {
                return CheckLine {
                    name: "配置安全",
                    status: CheckStatus::Warn,
                    detail: "项目 .yunxi/settings.local.json 含 apiKey，请迁到 ~/.yunxi/ 并确保已 gitignore"
                        .to_string(),
                };
            }
        }
    }

    if user_exists {
        CheckLine {
            name: "配置目录",
            status: CheckStatus::Ok,
            detail: format!(
                "用户配置 {}",
                user_local
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            ),
        }
    } else {
        CheckLine {
            name: "配置目录",
            status: CheckStatus::Warn,
            detail:
                "未找到 ~/.yunxi/settings.local.json；可复制 .yunxi/settings.semantic.example.json"
                    .to_string(),
        }
    }
}

fn check_patent_kg_canonical(repo_root: Option<&Path>) -> CheckLine {
    let Some(root) = repo_root else {
        return CheckLine {
            name: "专利图谱路径",
            status: CheckStatus::Ok,
            detail: "运行时在仓库外，使用 KnowledgePaths::discover()".to_string(),
        };
    };
    let canonical = root.join("assets/knowledge_graph/patent_kg.db");
    let alt = root.join("assets/knowledge-base/patent_kg.db");
    if !alt.is_file() {
        return CheckLine {
            name: "专利图谱路径",
            status: if canonical.is_file() {
                CheckStatus::Ok
            } else {
                CheckStatus::Warn
            },
            detail: canonical.display().to_string(),
        };
    }
    let canon_meta = std::fs::metadata(&canonical).ok();
    let alt_meta = std::fs::metadata(&alt).ok();
    let newer_alt = match (canon_meta, alt_meta) {
        (Some(c), Some(a)) => a.modified().ok() > c.modified().ok(),
        (None, Some(_)) => true,
        _ => false,
    };
    if newer_alt {
        CheckLine {
            name: "专利图谱路径",
            status: CheckStatus::Warn,
            detail: format!(
                "knowledge-base/patent_kg.db 较新，请同步到唯一路径 {}",
                canonical.display()
            ),
        }
    } else {
        CheckLine {
            name: "专利图谱路径",
            status: CheckStatus::Ok,
            detail: format!("使用 {}", canonical.display()),
        }
    }
}

fn check_knowledge_assets(repo_root: Option<&Path>) -> Vec<CheckLine> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _ = repo_root;
    let paths = KnowledgePaths::discover();
    vec![
        asset_line("专利知识图谱", paths.patent_kg_db.as_deref()),
        asset_line("语义索引", paths.semantic_index_db.as_deref()),
        asset_line("法律法规库", paths.laws_db.as_deref()),
        CheckLine {
            name: "工作目录",
            status: CheckStatus::Ok,
            detail: cwd.display().to_string(),
        },
    ]
}

fn asset_line(label: &'static str, path: Option<&str>) -> CheckLine {
    match path {
        Some(p) if Path::new(p).is_file() => CheckLine {
            name: label,
            status: CheckStatus::Ok,
            detail: p.to_string(),
        },
        Some(p) => CheckLine {
            name: label,
            status: CheckStatus::Warn,
            detail: format!("路径已解析但文件不存在: {p}"),
        },
        None => CheckLine {
            name: label,
            status: CheckStatus::Warn,
            detail: "未找到；运行 scripts/sync-knowledge-base.sh 或设置 PATENT_KG_DB 等"
                .to_string(),
        },
    }
}

fn check_omlx() -> CheckLine {
    let output = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "http://127.0.0.1:8009/v1/embeddings",
        ])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let code = String::from_utf8_lossy(&out.stdout);
            let code = code.trim();
            if code == "000" || code.is_empty() {
                CheckLine {
                    name: "oMLX 嵌入服务",
                    status: CheckStatus::Warn,
                    detail: "无法连接 127.0.0.1:8009；混合语义检索不可用".to_string(),
                }
            } else {
                CheckLine {
                    name: "oMLX 嵌入服务",
                    status: CheckStatus::Ok,
                    detail: format!("127.0.0.1:8009 可达 (HTTP {code})"),
                }
            }
        }
        _ => CheckLine {
            name: "oMLX 嵌入服务",
            status: CheckStatus::Warn,
            detail: "无法探测 127.0.0.1:8009；请启动 oMLX 或配置 semantic.http".to_string(),
        },
    }
}

fn check_patent_tools() -> Vec<CheckLine> {
    let markitdown = Command::new("python3")
        .args(["-c", "import markitdown"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let pdftoppm = Command::new("which")
        .arg("pdftoppm")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    vec![
        CheckLine {
            name: "markitdown",
            status: if markitdown {
                CheckStatus::Ok
            } else {
                CheckStatus::Warn
            },
            detail: if markitdown {
                "python3 -c 'import markitdown' 通过".to_string()
            } else {
                "未安装；运行 pip install 'markitdown[pdf,docx,pptx]'".to_string()
            },
        },
        CheckLine {
            name: "poppler (pdftoppm)",
            status: if pdftoppm {
                CheckStatus::Ok
            } else {
                CheckStatus::Warn
            },
            detail: if pdftoppm {
                "pdftoppm 可用".to_string()
            } else {
                "未安装；扫描 PDF OCR 需 brew install poppler".to_string()
            },
        },
    ]
}

fn check_rust_tests_hint() -> CheckLine {
    CheckLine {
        name: "开发验证",
        status: CheckStatus::Ok,
        detail: "cd rust && cargo test --workspace && cd .. && python3 -m pytest tests/ -v"
            .to_string(),
    }
}

fn locate_repo_root() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join("rust/Cargo.toml").is_file() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}
