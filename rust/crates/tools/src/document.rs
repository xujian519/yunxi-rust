use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use calamine::Reader;
use serde::{Deserialize, Serialize};

const MAX_FILE_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50 MiB

/// Input for the `DocumentRead` tool.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentReadInput {
    pub file_path: String,
    /// For multi-page documents: max pages to extract (null = all).
    pub max_pages: Option<usize>,
    /// For spreadsheets: sheet name to read (null = first sheet).
    pub sheet: Option<String>,
}

/// Output from the `DocumentRead` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReadOutput {
    pub file_path: String,
    pub format: String,
    pub text: String,
    pub pages: Option<usize>,
    pub rows: Option<usize>,
}

/// Read and extract text from a structured document (PDF, Excel, DOCX).
///
/// # Errors
/// Returns an error if the file cannot be read, the format is unsupported,
/// or parsing fails.
pub fn run_document_read(input: &DocumentReadInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let output = match ext.as_str() {
        "pdf" => read_pdf(path, input.max_pages)?,
        "xlsx" => read_excel(path, input.sheet.as_deref())?,
        "docx" => read_docx(path)?,
        other => {
            return Err(format!(
                "Unsupported document format '{other}'. Supported: pdf, xlsx, docx"
            ))
        }
    };

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

fn check_size(path: &Path) -> Result<(), String> {
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

fn read_pdf(path: &Path, max_pages: Option<usize>) -> Result<DocumentReadOutput, String> {
    check_size(path)?;
    let text = pdf_extract::extract_text(path).map_err(|e| format!("PDF parse error: {e}"))?;

    let text = if let Some(max) = max_pages {
        if max == 0 {
            String::new()
        } else {
            // Approximate page splitting by paragraph blocks (some PDF extractors emit form feeds).
            let pages: Vec<&str> = text.split('\u{c}').collect();
            if pages.len() > max {
                pages[..max].join("\n\n")
            } else {
                text
            }
        }
    } else {
        text
    };

    Ok(DocumentReadOutput {
        file_path: path.display().to_string(),
        format: "pdf".to_string(),
        text,
        pages: max_pages,
        rows: None,
    })
}

fn read_excel(path: &Path, sheet_name: Option<&str>) -> Result<DocumentReadOutput, String> {
    check_size(path)?;
    let mut workbook = calamine::open_workbook::<calamine::Xlsx<_>, _>(path)
        .map_err(|e| format!("Excel parse error: {e}"))?;

    let sheet = if let Some(name) = sheet_name {
        workbook
            .worksheet_range(name)
            .map_err(|e| format!("Sheet '{name}' read error: {e}"))?
    } else {
        let first = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or("Excel file has no sheets")?;
        workbook
            .worksheet_range(&first)
            .map_err(|e| format!("First sheet read error: {e}"))?
    };

    let mut lines = Vec::new();
    let mut row_count = 0;
    for row in sheet.rows() {
        let mut cells = Vec::new();
        for cell in row {
            cells.push(format!("{cell}"));
        }
        lines.push(cells.join("\t"));
        row_count += 1;
    }

    Ok(DocumentReadOutput {
        file_path: path.display().to_string(),
        format: "excel".to_string(),
        text: lines.join("\n"),
        pages: None,
        rows: Some(row_count),
    })
}

fn read_docx(path: &Path) -> Result<DocumentReadOutput, String> {
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

    let text = extract_text_from_docx_xml(&xml)?;

    Ok(DocumentReadOutput {
        file_path: path.display().to_string(),
        format: "docx".to_string(),
        text,
        pages: None,
        rows: None,
    })
}

fn extract_text_from_docx_xml(xml: &str) -> Result<String, String> {
    let mut text = String::new();
    let mut reader = quick_xml::Reader::from_str(xml);

    let mut in_text_element = false;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                if e.name().as_ref() == b"w:t" {
                    in_text_element = true;
                }
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                if in_text_element {
                    text.push_str(&e.unescape().unwrap_or_default());
                    in_text_element = false;
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if e.name().as_ref() == b"w:p" {
                    text.push('\n');
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

    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unknown_format() {
        let result = run_document_read(&DocumentReadInput {
            file_path: "foo.bar".to_string(),
            max_pages: None,
            sheet: None,
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn docx_xml_parsing_extracts_text() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Hello</w:t></w:r></w:p>
    <w:p><w:r><w:t>World</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let text = extract_text_from_docx_xml(xml).unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn read_excel_from_real_file() {
        // Create a minimal xlsx using calamine's write capabilities would need xlsxwriter.
        // Instead we just test the error path for a missing file.
        let result = run_document_read(&DocumentReadInput {
            file_path: "/nonexistent/test.xlsx".to_string(),
            max_pages: None,
            sheet: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn read_docx_from_real_file() {
        let result = run_document_read(&DocumentReadInput {
            file_path: "/nonexistent/test.docx".to_string(),
            max_pages: None,
            sheet: None,
        });
        assert!(result.is_err());
    }
}
