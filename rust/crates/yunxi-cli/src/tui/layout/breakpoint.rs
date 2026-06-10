//! 三断点响应式布局系统。
//!
//! Wide (>=160): 全功能布局
//! Standard (80-159): 标准布局
//! Compact (<80): 紧凑布局

/// 视口断点枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewport {
    Wide,     // >=160 字符
    Standard, // 80-159 字符
    Compact,  // <80 字符
}

impl Viewport {
    /// 根据宽度判定断点。
    pub fn from_size(width: u16) -> Self {
        if width >= 160 {
            Viewport::Wide
        } else if width >= 80 {
            Viewport::Standard
        } else {
            Viewport::Compact
        }
    }

    /// 侧边栏是否可见。
    pub fn sidebar_visible(&self) -> bool {
        matches!(self, Viewport::Wide)
    }

    /// 工具面板宽度（None 表示隐藏）。
    pub fn tool_panel_width(&self, total_width: u16) -> Option<u16> {
        match self {
            Viewport::Wide => Some((total_width * 15 / 100).max(20)),
            Viewport::Standard => Some((total_width * 35 / 100).max(20)),
            Viewport::Compact => None,
        }
    }

    /// 是否为紧凑模式。
    pub fn is_compact(&self) -> bool {
        matches!(self, Viewport::Compact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_breakpoint() {
        assert_eq!(Viewport::from_size(160), Viewport::Wide);
        assert_eq!(Viewport::from_size(200), Viewport::Wide);
    }

    #[test]
    fn standard_breakpoint() {
        assert_eq!(Viewport::from_size(80), Viewport::Standard);
        assert_eq!(Viewport::from_size(120), Viewport::Standard);
        assert_eq!(Viewport::from_size(159), Viewport::Standard);
    }

    #[test]
    fn compact_breakpoint() {
        assert_eq!(Viewport::from_size(79), Viewport::Compact);
        assert_eq!(Viewport::from_size(40), Viewport::Compact);
        assert_eq!(Viewport::from_size(0), Viewport::Compact);
    }

    #[test]
    fn sidebar_only_on_wide() {
        assert!(Viewport::Wide.sidebar_visible());
        assert!(!Viewport::Standard.sidebar_visible());
        assert!(!Viewport::Compact.sidebar_visible());
    }

    #[test]
    fn tool_panel_widths() {
        assert!(Viewport::Compact.tool_panel_width(100).is_none());
        assert_eq!(Viewport::Standard.tool_panel_width(100), Some(35));
        assert_eq!(Viewport::Wide.tool_panel_width(200), Some(30));
        // 最小 20
        assert_eq!(Viewport::Wide.tool_panel_width(100), Some(20));
    }

    #[test]
    fn is_compact_flag() {
        assert!(Viewport::Compact.is_compact());
        assert!(!Viewport::Standard.is_compact());
        assert!(!Viewport::Wide.is_compact());
    }
}
