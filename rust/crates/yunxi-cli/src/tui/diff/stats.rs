#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffStats {
    pub added: usize,
    pub deleted: usize,
    pub modified: usize,
}

impl DiffStats {
    pub fn new(added: usize, deleted: usize, modified: usize) -> Self {
        Self {
            added,
            deleted,
            modified,
        }
    }

    pub fn total(&self) -> usize {
        self.added + self.deleted + self.modified
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

impl Default for DiffStats {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_stats_total() {
        let stats = DiffStats::new(5, 3, 2);
        assert_eq!(stats.total(), 10);
    }

    #[test]
    fn test_diff_stats_empty() {
        let stats = DiffStats::default();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_diff_stats_non_empty() {
        let stats = DiffStats::new(1, 0, 0);
        assert!(!stats.is_empty());
    }
}
