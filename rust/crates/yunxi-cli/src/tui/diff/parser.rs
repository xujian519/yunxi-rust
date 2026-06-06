use crate::tui::diff::stats::DiffStats;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffChange {
    Added(String),
    Deleted(String),
    Modified { old: String, new: String },
    Unchanged(String),
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone)]
pub struct DiffParser {
    hunks: Vec<DiffHunk>,
}

impl DiffParser {
    pub fn new() -> Self {
        Self { hunks: Vec::new() }
    }

    pub fn parse_diff(&mut self, diff_text: &str) -> Result<(), String> {
        self.hunks.clear();
        let lines: Vec<&str> = diff_text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with("@@") {
                match self.parse_hunk_header(line) {
                    Ok((old_start, old_lines, new_start, new_lines)) => {
                        let mut changes = Vec::new();
                        i += 1;

                        let mut old_remaining = old_lines;
                        let mut new_remaining = new_lines;

                        while i < lines.len() && (old_remaining > 0 || new_remaining > 0) {
                            let change_line = lines[i];

                            if let Some(rest) = change_line.strip_prefix('+') {
                                changes.push(DiffChange::Added(rest.to_string()));
                                new_remaining = new_remaining.saturating_sub(1);
                            } else if let Some(rest) = change_line.strip_prefix('-') {
                                changes.push(DiffChange::Deleted(rest.to_string()));
                                old_remaining = old_remaining.saturating_sub(1);
                            } else if let Some(rest) = change_line.strip_prefix(' ') {
                                changes.push(DiffChange::Unchanged(rest.to_string()));
                                old_remaining = old_remaining.saturating_sub(1);
                                new_remaining = new_remaining.saturating_sub(1);
                            } else if change_line.starts_with('\\') {
                                i += 1;
                                continue;
                            } else {
                                break;
                            }
                            i += 1;
                        }

                        self.hunks.push(DiffHunk {
                            old_start,
                            old_lines,
                            new_start,
                            new_lines,
                            changes,
                        });
                    }
                    Err(e) => return Err(e),
                }
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    fn parse_hunk_header(&self, line: &str) -> Result<(usize, usize, usize, usize), String> {
        let header = line.trim_start_matches("@@").trim_end_matches("@@");

        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(format!("Invalid hunk header: {}", line));
        }

        let old_range = parts[0].trim_start_matches('-');
        let new_range = parts[1].trim_start_matches('+');

        let parse_range = |range: &str| -> Result<(usize, usize), String> {
            let nums: Vec<&str> = range.split(',').collect();
            let start = nums[0]
                .parse::<usize>()
                .map_err(|_| format!("Invalid start line: {}", nums[0]))?;
            let count = if nums.len() > 1 {
                nums[1]
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid line count: {}", nums[1]))?
            } else {
                1
            };
            Ok((start, count))
        };

        let (old_start, old_lines) = parse_range(old_range)?;
        let (new_start, new_lines) = parse_range(new_range)?;

        Ok((old_start, old_lines, new_start, new_lines))
    }

    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    pub fn stats(&self) -> DiffStats {
        let mut added = 0;
        let mut deleted = 0;
        let mut modified = 0;

        for hunk in &self.hunks {
            for change in &hunk.changes {
                match change {
                    DiffChange::Added(_) => added += 1,
                    DiffChange::Deleted(_) => deleted += 1,
                    DiffChange::Modified { .. } => modified += 1,
                    DiffChange::Unchanged(_) => {}
                }
            }
        }

        DiffStats {
            added,
            deleted,
            modified,
        }
    }
}

impl Default for DiffParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff = r"--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,4 @@
 line 1
-line 2
+line 2 modified
 line 3
+line 4";
        let mut parser = DiffParser::new();
        assert!(parser.parse_diff(diff).is_ok());
        assert_eq!(parser.hunks().len(), 1);
    }

    #[test]
    fn test_parse_hunk_header() {
        let parser = DiffParser::new();
        let (old_start, old_lines, new_start, new_lines) =
            parser.parse_hunk_header("@@ -1,3 +1,4 @@").unwrap();
        assert_eq!(old_start, 1);
        assert_eq!(old_lines, 3);
        assert_eq!(new_start, 1);
        assert_eq!(new_lines, 4);
    }

    #[test]
    fn test_stats_calculation() {
        let diff = r"@@ -1,3 +1,4 @@
 line 1
-line 2
+line 2 modified
 line 3
+line 4";
        let mut parser = DiffParser::new();
        parser.parse_diff(diff).unwrap();
        let stats = parser.stats();
        assert_eq!(stats.added, 2);
        assert_eq!(stats.deleted, 1);
    }
}
