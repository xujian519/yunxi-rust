use std::collections::HashMap;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use calamine::Reader;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

const MAX_FILE_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50 MiB

// ============================================================================
// PdfParse - Enhanced PDF Parser
// ============================================================================

/// Input for the `PdfParse` tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfParseInput {
    pub file_path: String,
    #[serde(default = "default_operation")]
    pub operation: String,
    #[allow(dead_code)]
    pub start_page: Option<u32>, // 保留原因: 预留给 PDF 分页提取功能
    #[allow(dead_code)]
    pub end_page: Option<u32>, // 保留原因: 预留给 PDF 分页提取功能
}

fn default_operation() -> String {
    "extract_text".to_string()
}

/// Output from the `PdfParse` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfParseOutput {
    pub file_path: String,
    pub operation: String,
    pub text: String,
    pub page_count: usize,
    pub word_count: usize,
    pub char_count: usize,
    pub pages: Vec<PdfPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPage {
    pub page_number: usize,
    pub text: String,
    pub word_count: usize,
}

/// Enhanced PDF parser with multiple operation modes.
pub fn run_pdf_parse(input: &PdfParseInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);
    check_size(path)?;

    let full_text = pdf_extract::extract_text(path).map_err(|e| format!("PDF parse error: {e}"))?;

    let char_count = full_text.chars().count();
    let output = match input.operation.as_str() {
        "extract_text" => {
            let word_count = full_text.split_whitespace().count();
            PdfParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text: full_text,
                page_count: 1,
                word_count,
                char_count,
                pages: vec![],
            }
        }
        "parse" => {
            let pages_raw: Vec<&str> = full_text.split('\u{c}').collect();
            let mut pages = Vec::new();
            let mut total_words = 0;

            for (idx, page_text) in pages_raw.iter().enumerate() {
                let word_count = page_text.split_whitespace().count();
                total_words += word_count;
                pages.push(PdfPage {
                    page_number: idx + 1,
                    text: page_text.to_string(),
                    word_count,
                });
            }

            PdfParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text: full_text,
                page_count: pages.len(),
                word_count: total_words,
                char_count,
                pages,
            }
        }
        "to_markdown" => {
            let pages_raw: Vec<&str> = full_text.split('\u{c}').collect();
            let mut md = String::new();
            let mut total_words = 0;

            for (idx, page_text) in pages_raw.iter().enumerate() {
                let word_count = page_text.split_whitespace().count();
                total_words += word_count;
                let _ = write!(md, "\n\n--- Page {} ---\n\n", idx + 1);
                md.push_str(page_text);
            }

            PdfParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text: md,
                page_count: pages_raw.len(),
                word_count: total_words,
                char_count,
                pages: vec![],
            }
        }
        _ => return Err(format!("Unknown operation: {}", input.operation)),
    };

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

// ============================================================================
// DocxParse - Enhanced DOCX Parser
// ============================================================================

/// Input for the `DocxParse` tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocxParseInput {
    pub file_path: String,
    #[serde(default = "default_docx_operation")]
    pub operation: String,
}

fn default_docx_operation() -> String {
    "extract_text".to_string()
}

/// Output from the `DocxParse` tool.
#[derive(Debug, Clone, Serialize)]
pub struct DocxParseOutput {
    pub file_path: String,
    pub operation: String,
    pub text: String,
    pub format: String,
    pub paragraph_count: usize,
    pub word_count: usize,
    pub sections: Vec<String>,
}

/// Enhanced DOCX parser with multiple output formats.
pub fn run_docx_parse(input: &DocxParseInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);
    check_size(path)?;

    let file = File::open(path).map_err(|e| format!("Cannot open DOCX file: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(BufReader::new(file)).map_err(|e| format!("ZIP open error: {e}"))?;

    let mut xml = String::new();
    {
        let mut entry = archive
            .by_name("word/document.xml")
            .map_err(|e| format!("DOCX missing word/document.xml: {e}"))?;
        entry
            .read_to_string(&mut xml)
            .map_err(|e| format!("DOCX read error: {e}"))?;
    }

    let paragraphs = extract_paragraphs_from_docx_xml(&xml)?;

    let output = match input.operation.as_str() {
        "extract_text" => {
            let text = paragraphs.join("\n\n");
            let word_count = text.split_whitespace().count();
            DocxParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text,
                format: "docx".to_string(),
                paragraph_count: paragraphs.len(),
                word_count,
                sections: vec![],
            }
        }
        "to_html" => {
            let html = paragraphs
                .iter()
                .map(|p| format!("<p>{p}</p>"))
                .collect::<Vec<_>>()
                .join("\n");
            let word_count = paragraphs.join(" ").split_whitespace().count();
            DocxParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text: html,
                format: "html".to_string(),
                paragraph_count: paragraphs.len(),
                word_count,
                sections: vec![],
            }
        }
        "to_markdown" => {
            let md = paragraphs.join("\n\n");
            let word_count = md.split_whitespace().count();
            DocxParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text: md,
                format: "markdown".to_string(),
                paragraph_count: paragraphs.len(),
                word_count,
                sections: vec![],
            }
        }
        "parse" => {
            let text = paragraphs.join("\n\n");
            let word_count = text.split_whitespace().count();
            let sections = detect_patent_sections(&paragraphs);
            DocxParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                text,
                format: "docx".to_string(),
                paragraph_count: paragraphs.len(),
                word_count,
                sections,
            }
        }
        _ => return Err(format!("Unknown operation: {}", input.operation)),
    };

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

fn extract_paragraphs_from_docx_xml(xml: &str) -> Result<Vec<String>, String> {
    let mut paragraphs = Vec::new();
    let mut current_paragraph = String::new();
    let mut reader = quick_xml::Reader::from_str(xml);

    let mut in_text_element = false;
    let mut in_bold = false;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                if e.name().as_ref() == b"w:t" {
                    in_text_element = true;
                } else if e.name().as_ref() == b"w:b" || e.name().as_ref() == b"w:bCs" {
                    in_bold = true;
                }
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                if in_text_element {
                    let text = e.unescape().unwrap_or_default();
                    if in_bold {
                        let _ = write!(current_paragraph, "**{text}**");
                    } else {
                        current_paragraph.push_str(&text);
                    }
                    in_text_element = false;
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if e.name().as_ref() == b"w:p" {
                    if !current_paragraph.is_empty() {
                        paragraphs.push(current_paragraph.clone());
                        current_paragraph = String::new();
                    }
                } else if e.name().as_ref() == b"w:b" || e.name().as_ref() == b"w:bCs" {
                    in_bold = false;
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parse error at position {}: {e}",
                    reader.buffer_position()
                ))
            }
            _ => {}
        }
    }

    Ok(paragraphs)
}

fn detect_patent_sections(paragraphs: &[String]) -> Vec<String> {
    let mut sections = Vec::new();
    let section_keywords = [
        "技术领域",
        "背景技术",
        "发明内容",
        "权利要求书",
        "摘要",
        "具体实施方式",
        "附图说明",
    ];

    for para in paragraphs {
        for keyword in &section_keywords {
            if para.contains(keyword) && !sections.contains(&keyword.to_string()) {
                sections.push(keyword.to_string());
            }
        }
    }

    sections
}

// ============================================================================
// ExcelParse - Enhanced Excel Parser
// ============================================================================

/// Input for the `ExcelParse` tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcelParseInput {
    pub file_path: String,
    #[serde(default = "default_excel_operation")]
    pub operation: String,
    pub sheet_name: Option<String>,
    #[serde(default = "default_max_rows")]
    pub max_rows: Option<usize>,
}

fn default_excel_operation() -> String {
    "read".to_string()
}

#[allow(clippy::unnecessary_wraps)]
fn default_max_rows() -> Option<usize> {
    Some(1000)
}

/// Output from the `ExcelParse` tool.
#[derive(Debug, Clone, Serialize)]
pub struct ExcelParseOutput {
    pub file_path: String,
    pub operation: String,
    pub sheet_name: String,
    pub available_sheets: Vec<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
}

/// Enhanced Excel parser with multiple output formats.
#[allow(clippy::too_many_lines)]
pub fn run_excel_parse(input: &ExcelParseInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);
    check_size(path)?;

    let mut workbook =
        calamine::open_workbook_auto(path).map_err(|e| format!("Excel parse error: {e}"))?;

    let sheet_names = workbook.sheet_names().clone();
    let target_sheet = if let Some(name) = &input.sheet_name {
        if !sheet_names.contains(name) {
            return Err(format!("Sheet '{name}' not found"));
        }
        name.clone()
    } else {
        sheet_names
            .first()
            .cloned()
            .ok_or("Excel file has no sheets")?
    };

    let range = workbook
        .worksheet_range(&target_sheet)
        .map_err(|e| format!("Sheet '{target_sheet}' read error: {e}"))?;

    let mut rows = Vec::new();
    let max_rows = input.max_rows.unwrap_or(1000);

    for (idx, row) in range.rows().enumerate() {
        if idx >= max_rows {
            break;
        }
        let cells: Vec<String> = row.iter().map(|cell| format!("{cell}")).collect();
        rows.push(cells);
    }

    let headers = rows.first().cloned().unwrap_or_default();
    let column_count = headers.len();
    let row_count = rows.len();

    let output = match input.operation.as_str() {
        "read" => {
            let _text: Vec<String> = rows.iter().map(|row| row.join("\t")).collect();
            ExcelParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                sheet_name: target_sheet,
                available_sheets: sheet_names,
                headers,
                rows,
                row_count,
                column_count,
            }
        }
        "to_json" => {
            let _json_rows: Vec<serde_json::Value> = if rows.len() > 1 {
                let header = &rows[0];
                rows[1..]
                    .iter()
                    .map(|row| {
                        let mut obj = serde_json::Map::new();
                        for (idx, val) in row.iter().enumerate() {
                            if idx < header.len() {
                                obj.insert(header[idx].clone(), json!(val));
                            }
                        }
                        json!(obj)
                    })
                    .collect()
            } else {
                vec![]
            };
            ExcelParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                sheet_name: target_sheet,
                available_sheets: sheet_names,
                headers,
                rows,
                row_count,
                column_count,
            }
        }
        "to_markdown" => {
            let _md_table = if rows.is_empty() {
                String::new()
            } else {
                let header = &rows[0];
                let mut md = format!("| {} |\n", header.join(" | "));
                let _ = writeln!(md, "| {} |", vec!["---"; header.len()].join(" | "));
                for row in &rows[1..] {
                    let _ = writeln!(md, "| {} |", row.join(" | "));
                }
                md
            };
            ExcelParseOutput {
                file_path: input.file_path.clone(),
                operation: input.operation.clone(),
                sheet_name: target_sheet,
                available_sheets: sheet_names,
                headers,
                rows,
                row_count,
                column_count,
            }
        }
        "parse" => ExcelParseOutput {
            file_path: input.file_path.clone(),
            operation: input.operation.clone(),
            sheet_name: target_sheet,
            available_sheets: sheet_names,
            headers,
            rows,
            row_count,
            column_count,
        },
        _ => return Err(format!("Unknown operation: {}", input.operation)),
    };

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

// ============================================================================
// MarkdownParse - Patent Document Section Parser
// ============================================================================

/// Input for the `MarkdownParse` tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownParseInput {
    pub text: String,
    #[serde(default = "default_markdown_operation")]
    pub operation: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub metadata: Option<HashMap<String, String>>, // 保留原因: 预留给元数据处理和增强输出
}

fn default_markdown_operation() -> String {
    "parse_markdown".to_string()
}

/// Output from the `MarkdownParse` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownParseOutput {
    pub operation: String,
    pub sections: Vec<PatentSection>,
    pub section_count: usize,
    pub total_chars: usize,
    pub claims_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatentSection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<usize>,
}

/// Patent document parser with multiple parsing modes.
pub fn run_markdown_parse(input: &MarkdownParseInput) -> Result<String, String> {
    let output = match input.operation.as_str() {
        "parse_markdown" => parse_markdown_sections(&input.text),
        "parse_plain_text" => Ok(parse_plain_text_sections(&input.text)),
        "parse_claims" => parse_claims(&input.text),
        "parse_opinion_statement" => parse_opinion_statement(&input.text),
        _ => return Err(format!("Unknown operation: {}", input.operation)),
    }?;

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

fn parse_markdown_sections(text: &str) -> Result<MarkdownParseOutput, String> {
    let section_aliases: HashMap<&str, Vec<&str>> = [
        ("title", vec!["标题", "发明名称", "title"]),
        ("technical_field", vec!["技术领域", "technical field"]),
        ("background", vec!["背景技术", "现有技术", "background"]),
        ("invention_summary", vec!["发明内容", "发明目的", "summary"]),
        ("claims", vec!["权利要求书", "claims"]),
        ("abstract", vec!["摘要", "abstract"]),
        ("embodiments", vec!["具体实施方式", "实施例", "embodiments"]),
        ("drawings", vec!["附图说明", "drawings"]),
    ]
    .iter()
    .cloned()
    .collect();

    let heading_re = Regex::new(r"^#{1,3}\s+(.+)$").map_err(|e| e.to_string())?;
    let mut sections = Vec::new();
    let mut current_section = PatentSection {
        name: "intro".to_string(),
        aliases: None,
        content: String::new(),
        level: None,
    };

    for line in text.lines() {
        if let Some(caps) = heading_re.captures(line) {
            if !current_section.content.trim().is_empty() {
                sections.push(current_section);
            }
            let heading = caps.get(1).map_or("", |m| m.as_str());
            let level = line.chars().take_while(|c| *c == '#').count();
            current_section = PatentSection {
                name: normalize_section_name(heading, &section_aliases),
                aliases: Some(vec![heading.to_string()]),
                content: String::new(),
                level: Some(level),
            };
        } else {
            current_section.content.push_str(line);
            current_section.content.push('\n');
        }
    }

    if !current_section.content.trim().is_empty() {
        sections.push(current_section);
    }

    let total_chars = text.chars().count();
    let section_count = sections.len();
    let claims_count = sections
        .iter()
        .find(|s| s.name == "claims")
        .map(|s| s.content.matches("权利要求").count());

    Ok(MarkdownParseOutput {
        operation: "parse_markdown".to_string(),
        sections,
        section_count,
        total_chars,
        claims_count,
    })
}

fn normalize_section_name(heading: &str, aliases: &HashMap<&str, Vec<&str>>) -> String {
    for (name, alias_list) in aliases {
        if alias_list.iter().any(|&alias| heading.contains(alias)) {
            return name.to_string();
        }
    }
    heading.to_lowercase().replace(' ', "_")
}

fn parse_plain_text_sections(text: &str) -> MarkdownParseOutput {
    let section_markers = [
        "技术领域：",
        "背景技术：",
        "发明内容：",
        "权利要求书：",
        "摘要：",
        "具体实施方式：",
        "附图说明：",
    ];

    let mut sections = Vec::new();
    let mut current_section = PatentSection {
        name: "intro".to_string(),
        aliases: None,
        content: String::new(),
        level: None,
    };

    let lines = text.lines();
    for line in lines {
        let mut found_marker = false;
        for marker in &section_markers {
            if let Some(pos) = line.find(marker) {
                if !current_section.content.trim().is_empty() {
                    sections.push(current_section);
                }
                let name = marker.trim_end_matches('：').to_string();
                let rest = line[pos + marker.len()..].to_string();
                current_section = PatentSection {
                    name: normalize_section_name(&name, &HashMap::new()),
                    aliases: Some(vec![name]),
                    content: rest,
                    level: None,
                };
                found_marker = true;
                break;
            }
        }

        if !found_marker {
            current_section.content.push_str(line);
            current_section.content.push('\n');
        }
    }

    sections.push(current_section);

    let total_chars = text.chars().count();
    let section_count = sections.len();
    let claims_count = sections
        .iter()
        .find(|s| s.name == "claims")
        .map(|s| s.content.matches("权利要求").count());

    MarkdownParseOutput {
        operation: "parse_plain_text".to_string(),
        sections,
        section_count,
        total_chars,
        claims_count,
    }
}

fn parse_claims(text: &str) -> Result<MarkdownParseOutput, String> {
    let claim_re = Regex::new(r"(?m)^(?:\d+\.\s*|权利要求\d+\s*[：:]|（\d+）)(.+)$")
        .map_err(|e| e.to_string())?;
    let dependency_re = Regex::new(r"根据权利要求(\d+)").map_err(|e| e.to_string())?;

    let mut sections = Vec::new();
    let mut claim_count = 0;

    for caps in claim_re.captures_iter(text) {
        let claim_text = caps.get(1).map_or("", |m| m.as_str());
        claim_count += 1;

        let dependencies: Vec<String> = dependency_re
            .captures_iter(claim_text)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        sections.push(PatentSection {
            name: format!("claim_{claim_count}"),
            aliases: if dependencies.is_empty() {
                None
            } else {
                Some(dependencies)
            },
            content: claim_text.to_string(),
            level: None,
        });
    }

    let total_chars = text.chars().count();
    let section_count = sections.len();

    Ok(MarkdownParseOutput {
        operation: "parse_claims".to_string(),
        sections,
        section_count,
        total_chars,
        claims_count: Some(claim_count),
    })
}

fn parse_opinion_statement(text: &str) -> Result<MarkdownParseOutput, String> {
    let opinion_re = Regex::new(r"(?m)^(审查意见|申请人认为|对比文件\d+)[：:](.+)$")
        .map_err(|e| e.to_string())?;

    let mut sections = Vec::new();
    let mut current_section = PatentSection {
        name: "intro".to_string(),
        aliases: None,
        content: String::new(),
        level: None,
    };

    for line in text.lines() {
        if let Some(caps) = opinion_re.captures(line) {
            if !current_section.content.trim().is_empty() {
                sections.push(current_section);
            }
            let speaker = caps.get(1).map_or("", |m| m.as_str());
            let content = caps.get(2).map_or("", |m| m.as_str());
            current_section = PatentSection {
                name: speaker.to_string(),
                aliases: None,
                content: content.to_string(),
                level: None,
            };
        } else {
            current_section.content.push_str(line);
            current_section.content.push('\n');
        }
    }

    if !current_section.content.trim().is_empty() {
        sections.push(current_section);
    }

    let total_chars = text.chars().count();
    let section_count = sections.len();

    Ok(MarkdownParseOutput {
        operation: "parse_opinion_statement".to_string(),
        sections,
        section_count,
        total_chars,
        claims_count: None,
    })
}

// ============================================================================
// Utility Functions
// ============================================================================

fn check_size(path: &Path) -> Result<(), String> {
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Path traversal not allowed".to_string());
    }
    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Cannot read file metadata: {e}"))?;
    if metadata.len() > MAX_FILE_SIZE_BYTES {
        return Err(format!(
            "File exceeds maximum size of {} MiB",
            MAX_FILE_SIZE_BYTES / 1024 / 1024
        ));
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_parse_invalid_file() {
        let result = run_pdf_parse(&PdfParseInput {
            file_path: "/nonexistent/test.pdf".to_string(),
            operation: "extract_text".to_string(),
            start_page: None,
            end_page: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_docx_parse_invalid_file() {
        let result = run_docx_parse(&DocxParseInput {
            file_path: "/nonexistent/test.docx".to_string(),
            operation: "extract_text".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_excel_parse_invalid_file() {
        let result = run_excel_parse(&ExcelParseInput {
            file_path: "/nonexistent/test.xlsx".to_string(),
            operation: "read".to_string(),
            sheet_name: None,
            max_rows: Some(100),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_markdown_parse_sections() {
        let text = "
# 发明名称
智能控制系统

## 技术领域
本发明涉及自动化控制技术领域。

## 背景技术
现有技术存在控制精度不足的问题。

## 发明内容
本发明提供一种高精度控制方法。

## 权利要求书
1. 一种智能控制方法，其特征在于...

## 具体实施方式
如图1所示，实施例包括...
";

        let result = run_markdown_parse(&MarkdownParseInput {
            text: text.to_string(),
            operation: "parse_markdown".to_string(),
            metadata: None,
        });
        assert!(result.is_ok());

        let output: MarkdownParseOutput = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(output.operation, "parse_markdown");
        assert!(output.section_count >= 5);
        assert!(output.total_chars > 0);
    }

    #[test]
    fn test_markdown_parse_claims() {
        let text = "
1. 一种智能控制方法，包括步骤A、B、C。

2. 根据权利要求1所述的方法，其特征在于，步骤A包括预处理。

3. 根据权利要求2所述的方法，其特征在于，预处理包括数据清洗。

权利要求1. 一种控制系统，包括处理器和存储器。
";

        let result = run_markdown_parse(&MarkdownParseInput {
            text: text.to_string(),
            operation: "parse_claims".to_string(),
            metadata: None,
        });
        assert!(result.is_ok());

        let output: MarkdownParseOutput = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(output.operation, "parse_claims");
        assert!(output.section_count >= 3);
        assert!(output.claims_count.unwrap_or(0) >= 3);
    }

    #[test]
    fn test_markdown_parse_plain_text() {
        let text = "
技术领域：本发明涉及人工智能领域。
背景技术：现有技术存在效率低下的问题。
发明内容：本发明提供一种高效算法。
权利要求书：1. 一种算法，包括...
";

        let result = run_markdown_parse(&MarkdownParseInput {
            text: text.to_string(),
            operation: "parse_plain_text".to_string(),
            metadata: None,
        });
        assert!(result.is_ok());

        let output: MarkdownParseOutput = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(output.operation, "parse_plain_text");
        assert!(
            output.section_count >= 1,
            "section_count = {}, expected >= 1",
            output.section_count
        );
    }

    #[test]
    fn test_markdown_parse_opinion_statement() {
        let text = "
审查意见：本申请不具备新颖性，对比文件1公开了所有技术特征。

申请人认为：对比文件1未公开特征X，本申请具备新颖性。

审查意见：对比文件2给出了技术启示。

申请人认为：对比文件2属于不同技术领域，不存在结合启示。
";

        let result = run_markdown_parse(&MarkdownParseInput {
            text: text.to_string(),
            operation: "parse_opinion_statement".to_string(),
            metadata: None,
        });
        assert!(result.is_ok());

        let output: MarkdownParseOutput = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(output.operation, "parse_opinion_statement");
        assert!(output.section_count >= 3);
    }

    #[test]
    fn test_markdown_parse_invalid_operation() {
        let result = run_markdown_parse(&MarkdownParseInput {
            text: "test".to_string(),
            operation: "invalid_op".to_string(),
            metadata: None,
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown operation"));
    }
}
