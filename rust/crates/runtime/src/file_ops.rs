use std::cmp::Reverse;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use glob::Pattern;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// 文本文件载荷
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextFilePayload {
    /// 文件路径
    #[serde(rename = "filePath")]
    pub file_path: String,
    /// 文件内容
    pub content: String,
    /// 行数
    #[serde(rename = "numLines")]
    pub num_lines: usize,
    /// 起始行号
    #[serde(rename = "startLine")]
    pub start_line: usize,
    /// 总行数
    #[serde(rename = "totalLines")]
    pub total_lines: usize,
}

/// 读取文件输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadFileOutput {
    /// 类型
    #[serde(rename = "type")]
    pub kind: String,
    /// 文件内容
    pub file: TextFilePayload,
}

/// 结构化补丁块
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredPatchHunk {
    /// 旧起始行
    #[serde(rename = "oldStart")]
    pub old_start: usize,
    /// 旧行数
    #[serde(rename = "oldLines")]
    pub old_lines: usize,
    /// 新起始行
    #[serde(rename = "newStart")]
    pub new_start: usize,
    /// 新行数
    #[serde(rename = "newLines")]
    pub new_lines: usize,
    /// 行列表
    pub lines: Vec<String>,
}

/// 写入文件输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteFileOutput {
    /// 类型
    #[serde(rename = "type")]
    pub kind: String,
    /// 文件路径
    #[serde(rename = "filePath")]
    pub file_path: String,
    /// 文件内容
    pub content: String,
    /// 结构化补丁
    #[serde(rename = "structuredPatch")]
    pub structured_patch: Vec<StructuredPatchHunk>,
    /// 原始文件
    #[serde(rename = "originalFile")]
    pub original_file: Option<String>,
    /// Git 差异
    #[serde(rename = "gitDiff")]
    pub git_diff: Option<serde_json::Value>,
}

/// 编辑文件输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditFileOutput {
    /// 文件路径
    #[serde(rename = "filePath")]
    pub file_path: String,
    /// 旧字符串
    #[serde(rename = "oldString")]
    pub old_string: String,
    /// 新字符串
    #[serde(rename = "newString")]
    pub new_string: String,
    /// 原始文件
    #[serde(rename = "originalFile")]
    pub original_file: String,
    /// 结构化补丁
    #[serde(rename = "structuredPatch")]
    pub structured_patch: Vec<StructuredPatchHunk>,
    /// 用户是否修改
    #[serde(rename = "userModified")]
    pub user_modified: bool,
    /// 是否替换全部
    #[serde(rename = "replaceAll")]
    pub replace_all: bool,
    /// Git 差异
    #[serde(rename = "gitDiff")]
    pub git_diff: Option<serde_json::Value>,
}

/// 全局搜索输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobSearchOutput {
    /// 持续时间（毫秒）
    #[serde(rename = "durationMs")]
    pub duration_ms: u128,
    /// 文件数量
    #[serde(rename = "numFiles")]
    pub num_files: usize,
    /// 文件名列表
    pub filenames: Vec<String>,
    /// 是否截断
    pub truncated: bool,
}

/// 正则搜索输入
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrepSearchInput {
    /// 搜索模式
    pub pattern: String,
    /// 路径
    pub path: Option<String>,
    /// Glob 模式
    pub glob: Option<String>,
    /// 输出模式
    #[serde(rename = "output_mode")]
    pub output_mode: Option<String>,
    /// 上下文行数（前）
    #[serde(rename = "-B")]
    pub before: Option<usize>,
    /// 上下文行数（后）
    #[serde(rename = "-A")]
    pub after: Option<usize>,
    /// 上下文行数（短）
    #[serde(rename = "-C")]
    pub context_short: Option<usize>,
    /// 上下文行数
    pub context: Option<usize>,
    /// 显示行号
    #[serde(rename = "-n")]
    pub line_numbers: Option<bool>,
    /// 忽略大小写
    #[serde(rename = "-i")]
    pub case_insensitive: Option<bool>,
    /// 文件类型
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    /// 结果限制
    pub head_limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
    /// 多行模式
    pub multiline: Option<bool>,
}

/// 正则搜索输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrepSearchOutput {
    /// 模式
    pub mode: Option<String>,
    /// 文件数量
    #[serde(rename = "numFiles")]
    pub num_files: usize,
    /// 文件名列表
    pub filenames: Vec<String>,
    /// 内容
    pub content: Option<String>,
    /// 行数
    #[serde(rename = "numLines")]
    pub num_lines: Option<usize>,
    /// 匹配数
    #[serde(rename = "numMatches")]
    pub num_matches: Option<usize>,
    /// 应用的限制
    #[serde(rename = "appliedLimit")]
    pub applied_limit: Option<usize>,
    /// 应用的偏移
    #[serde(rename = "appliedOffset")]
    pub applied_offset: Option<usize>,
}

/// 读取文件内容
///
/// # Errors
///
/// - 如果路径规范化失败,返回 IO 错误
/// - 如果文件读取失败,返回 IO 错误
/// 读取文件
///
/// # 参数
/// - `path`: 文件路径
/// - `offset`: 起始行偏移
/// - `limit`: 最大行数
///
/// # 返回
/// 读取文件输出
///
/// # 错误
/// - 如果路径无效，返回错误
/// - 如果文件读取失败，返回错误
pub fn read_file(
    path: &str,
    offset: Option<usize>,
    limit: Option<usize>,
) -> io::Result<ReadFileOutput> {
    let absolute_path = normalize_path(path)?;
    let content = fs::read_to_string(&absolute_path)?;
    let lines: Vec<&str> = content.lines().collect();
    let start_index = offset.unwrap_or(0).min(lines.len());
    let end_index = limit.map_or(lines.len(), |limit| {
        start_index.saturating_add(limit).min(lines.len())
    });
    let selected = lines[start_index..end_index].join("\n");

    Ok(ReadFileOutput {
        kind: String::from("text"),
        file: TextFilePayload {
            file_path: absolute_path.to_string_lossy().into_owned(),
            content: selected,
            num_lines: end_index.saturating_sub(start_index),
            start_line: start_index.saturating_add(1),
            total_lines: lines.len(),
        },
    })
}

/// 写入文件
///
/// # 参数
/// - `path`: 文件路径
/// - `content`: 文件内容
///
/// # 返回
/// 写入文件输出
///
/// # Errors
///
/// - 如果路径规范化失败,返回 IO 错误
/// - 如果目录创建失败,返回 IO 错误
/// - 如果文件写入失败,返回 IO 错误
pub fn write_file(path: &str, content: &str) -> io::Result<WriteFileOutput> {
    let absolute_path = normalize_path_allow_missing(path)?;
    let original_file = fs::read_to_string(&absolute_path).ok();
    if let Some(parent) = absolute_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&absolute_path, content)?;

    Ok(WriteFileOutput {
        kind: if original_file.is_some() {
            String::from("update")
        } else {
            String::from("create")
        },
        file_path: absolute_path.to_string_lossy().into_owned(),
        content: content.to_owned(),
        structured_patch: make_patch(original_file.as_deref().unwrap_or(""), content),
        original_file,
        git_diff: None,
    })
}

/// 编辑文件内容
///
/// # Errors
///
/// - 如果路径规范化失败,返回 IO 错误
/// - 如果文件读取失败,返回 IO 错误
/// - 如果 old_string 和 new_string 相同,返回错误
/// - 如果 old_string 未找到,返回错误
/// - 如果文件写入失败,返回 IO 错误
/// 编辑文件
///
/// # 参数
/// - `path`: 文件路径
/// - `old_string`: 旧字符串
/// - `new_string`: 新字符串
/// - `replace_all`: 是否替换全部
///
/// # 返回
/// 编辑文件输出
///
/// # 错误
/// - 如果路径无效，返回错误
/// - 如果文件读取失败，返回错误
/// - 如果旧字符串和新字符串相同，返回错误
/// - 如果旧字符串未找到，返回错误
pub fn edit_file(
    path: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> io::Result<EditFileOutput> {
    let absolute_path = normalize_path(path)?;
    let original_file = fs::read_to_string(&absolute_path)?;
    if old_string == new_string {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "old_string and new_string must differ",
        ));
    }
    if !original_file.contains(old_string) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "old_string not found in file",
        ));
    }

    let updated = if replace_all {
        original_file.replace(old_string, new_string)
    } else {
        original_file.replacen(old_string, new_string, 1)
    };
    fs::write(&absolute_path, &updated)?;

    Ok(EditFileOutput {
        file_path: absolute_path.to_string_lossy().into_owned(),
        old_string: old_string.to_owned(),
        new_string: new_string.to_owned(),
        original_file: original_file.clone(),
        structured_patch: make_patch(&original_file, &updated),
        user_modified: false,
        replace_all,
        git_diff: None,
    })
}

/// 全局搜索文件
///
/// # Errors
///
/// - 如果路径规范化失败,返回 IO 错误
/// - 如果目录访问失败,返回 IO 错误
/// - 如果文件枚举失败,返回 IO 错误
/// 全局搜索文件
///
/// # 参数
/// - `pattern`: Glob 模式
/// - `path`: 基础路径
///
/// # 返回
/// 全局搜索输出
///
/// # 错误
/// - 如果路径无效，返回错误
/// - 如果 Glob 模式无效，返回错误
pub fn glob_search(pattern: &str, path: Option<&str>) -> io::Result<GlobSearchOutput> {
    let started = Instant::now();
    let base_dir = path
        .map(normalize_path)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let search_pattern = if Path::new(pattern).is_absolute() {
        pattern.to_owned()
    } else {
        base_dir.join(pattern).to_string_lossy().into_owned()
    };

    let mut matches = Vec::new();
    let entries = glob::glob(&search_pattern)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    for entry in entries.flatten() {
        if entry.is_file() {
            matches.push(entry);
        }
    }

    matches.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .map(Reverse)
    });

    let truncated = matches.len() > 100;
    let filenames = matches
        .into_iter()
        .take(100)
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    Ok(GlobSearchOutput {
        duration_ms: started.elapsed().as_millis(),
        num_files: filenames.len(),
        filenames,
        truncated,
    })
}

/// 正则表达式搜索文件内容
///
/// # Errors
///
/// - 如果路径规范化失败,返回 IO 错误
/// - 如果当前目录访问失败,返回 IO 错误
/// - 如果正则表达式编译失败,返回 IO 错误
/// - 如果 glob 模式解析失败,返回 IO 错误
/// 正则搜索文件内容
///
/// # 参数
/// - `input`: 搜索输入
///
/// # 返回
/// 正则搜索输出
///
/// # 错误
/// - 如果路径无效，返回错误
/// - 如果正则表达式无效，返回错误
/// - 如果 Glob 模式无效，返回错误
pub fn grep_search(input: &GrepSearchInput) -> io::Result<GrepSearchOutput> {
    let base_path = input
        .path
        .as_deref()
        .map(normalize_path)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);

    let regex = RegexBuilder::new(&input.pattern)
        .case_insensitive(input.case_insensitive.unwrap_or(false))
        .dot_matches_new_line(input.multiline.unwrap_or(false))
        .build()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;

    let glob_filter = input
        .glob
        .as_deref()
        .map(Pattern::new)
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    let file_type = input.file_type.as_deref();
    let output_mode = input
        .output_mode
        .clone()
        .unwrap_or_else(|| String::from("files_with_matches"));
    let context = input.context.or(input.context_short).unwrap_or(0);

    let mut filenames = Vec::new();
    let mut content_lines = Vec::new();
    let mut total_matches = 0usize;

    for file_path in collect_search_files(&base_path)? {
        if !matches_optional_filters(&file_path, glob_filter.as_ref(), file_type) {
            continue;
        }

        let Ok(file_contents) = fs::read_to_string(&file_path) else {
            continue;
        };

        if output_mode == "count" {
            let count = regex.find_iter(&file_contents).count();
            if count > 0 {
                filenames.push(file_path.to_string_lossy().into_owned());
                total_matches += count;
            }
            continue;
        }

        let lines: Vec<&str> = file_contents.lines().collect();
        let mut matched_lines = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if regex.is_match(line) {
                total_matches += 1;
                matched_lines.push(index);
            }
        }

        if matched_lines.is_empty() {
            continue;
        }

        filenames.push(file_path.to_string_lossy().into_owned());
        if output_mode == "content" {
            for index in matched_lines {
                let start = index.saturating_sub(input.before.unwrap_or(context));
                let end = (index + input.after.unwrap_or(context) + 1).min(lines.len());
                for (current, line) in lines.iter().enumerate().take(end).skip(start) {
                    let prefix = if input.line_numbers.unwrap_or(true) {
                        format!("{}:{}:", file_path.to_string_lossy(), current + 1)
                    } else {
                        format!("{}:", file_path.to_string_lossy())
                    };
                    content_lines.push(format!("{prefix}{line}"));
                }
            }
        }
    }

    let (filenames, applied_limit, applied_offset) =
        apply_limit(filenames, input.head_limit, input.offset);
    let content_output = if output_mode == "content" {
        let (lines, limit, offset) = apply_limit(content_lines, input.head_limit, input.offset);
        return Ok(GrepSearchOutput {
            mode: Some(output_mode),
            num_files: filenames.len(),
            filenames,
            num_lines: Some(lines.len()),
            content: Some(lines.join("\n")),
            num_matches: None,
            applied_limit: limit,
            applied_offset: offset,
        });
    } else {
        None
    };

    Ok(GrepSearchOutput {
        mode: Some(output_mode.clone()),
        num_files: filenames.len(),
        filenames,
        content: content_output,
        num_lines: None,
        num_matches: (output_mode == "count").then_some(total_matches),
        applied_limit,
        applied_offset,
    })
}

fn collect_search_files(base_path: &Path) -> io::Result<Vec<PathBuf>> {
    if base_path.is_file() {
        return Ok(vec![base_path.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(base_path) {
        let entry = entry.map_err(|error| io::Error::other(error.to_string()))?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

fn matches_optional_filters(
    path: &Path,
    glob_filter: Option<&Pattern>,
    file_type: Option<&str>,
) -> bool {
    if let Some(glob_filter) = glob_filter {
        let path_string = path.to_string_lossy();
        if !glob_filter.matches(&path_string) && !glob_filter.matches_path(path) {
            return false;
        }
    }

    if let Some(file_type) = file_type {
        let extension = path.extension().and_then(|extension| extension.to_str());
        if extension != Some(file_type) {
            return false;
        }
    }

    true
}

fn apply_limit<T>(
    items: Vec<T>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> (Vec<T>, Option<usize>, Option<usize>) {
    let offset_value = offset.unwrap_or(0);
    let mut items = items.into_iter().skip(offset_value).collect::<Vec<_>>();
    let explicit_limit = limit.unwrap_or(250);
    if explicit_limit == 0 {
        return (items, None, (offset_value > 0).then_some(offset_value));
    }

    let truncated = items.len() > explicit_limit;
    items.truncate(explicit_limit);
    (
        items,
        truncated.then_some(explicit_limit),
        (offset_value > 0).then_some(offset_value),
    )
}

fn make_patch(original: &str, updated: &str) -> Vec<StructuredPatchHunk> {
    let mut lines = Vec::new();
    for line in original.lines() {
        lines.push(format!("-{line}"));
    }
    for line in updated.lines() {
        lines.push(format!("+{line}"));
    }

    vec![StructuredPatchHunk {
        old_start: 1,
        old_lines: original.lines().count(),
        new_start: 1,
        new_lines: updated.lines().count(),
        lines,
    }]
}

fn normalize_path(path: &str) -> io::Result<PathBuf> {
    let candidate = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?.join(path)
    };
    candidate.canonicalize()
}

fn normalize_path_allow_missing(path: &str) -> io::Result<PathBuf> {
    let candidate = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?.join(path)
    };

    if let Ok(canonical) = candidate.canonicalize() {
        return Ok(canonical);
    }

    if let Some(parent) = candidate.parent() {
        let canonical_parent = parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf());
        if let Some(name) = candidate.file_name() {
            return Ok(canonical_parent.join(name));
        }
    }

    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{edit_file, glob_search, grep_search, read_file, write_file, GrepSearchInput};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("yunxi-native-{name}-{unique}"))
    }

    #[test]
    fn reads_and_writes_files() {
        let path = temp_path("read-write.txt");
        let write_output = write_file(path.to_string_lossy().as_ref(), "one\ntwo\nthree")
            .expect("write should succeed");
        assert_eq!(write_output.kind, "create");

        let read_output = read_file(path.to_string_lossy().as_ref(), Some(1), Some(1))
            .expect("read should succeed");
        assert_eq!(read_output.file.content, "two");
    }

    #[test]
    fn edits_file_contents() {
        let path = temp_path("edit.txt");
        write_file(path.to_string_lossy().as_ref(), "alpha beta alpha")
            .expect("initial write should succeed");
        let output = edit_file(path.to_string_lossy().as_ref(), "alpha", "omega", true)
            .expect("edit should succeed");
        assert!(output.replace_all);
    }

    #[test]
    fn globs_and_greps_directory() {
        let dir = temp_path("search-dir");
        std::fs::create_dir_all(&dir).expect("directory should be created");
        let file = dir.join("demo.rs");
        write_file(
            file.to_string_lossy().as_ref(),
            "fn main() {\n println!(\"hello\");\n}\n",
        )
        .expect("file write should succeed");

        let globbed = glob_search("**/*.rs", Some(dir.to_string_lossy().as_ref()))
            .expect("glob should succeed");
        assert_eq!(globbed.num_files, 1);

        let grep_output = grep_search(&GrepSearchInput {
            pattern: String::from("hello"),
            path: Some(dir.to_string_lossy().into_owned()),
            glob: Some(String::from("**/*.rs")),
            output_mode: Some(String::from("content")),
            before: None,
            after: None,
            context_short: None,
            context: None,
            line_numbers: Some(true),
            case_insensitive: Some(false),
            file_type: None,
            head_limit: Some(10),
            offset: Some(0),
            multiline: Some(false),
        })
        .expect("grep should succeed");
        assert!(grep_output.content.unwrap_or_default().contains("hello"));
    }
}
