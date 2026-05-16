#![allow(dead_code)]

/// 布局区域定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub(crate) const ZERO: Self = Self {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    /// 从 (x, y) 和 (width, height) 构造。
    pub(crate) fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 区域面积（行数）。
    pub(crate) fn area(self) -> u32 {
        u32::from(self.width) * u32::from(self.height)
    }

    /// 内部可用区域（去掉边框 1px）。
    pub(crate) fn inner(self, margin: u16) -> Self {
        let double = margin.saturating_mul(2);
        Self {
            x: self.x.saturating_add(margin),
            y: self.y.saturating_add(margin),
            width: self.width.saturating_sub(double),
            height: self.height.saturating_sub(double),
        }
    }

    /// 是否为有效可见区域。
    pub(crate) fn is_valid(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// 全屏 TUI 布局计算。
///
/// ```text
/// ┌─────────────────────────────────────────────────┐
/// │ 顶部标题栏 (1行)                                │
/// ├──────────────────────────┬──────────────────────┤
/// │ 对话历史 (可滚动)        │ 工具输出面板         │
/// ├──────────────────────────┴──────────────────────┤
/// │ 输入框 (3行)                                    │
/// ├─────────────────────────────────────────────────┤
/// │ 状态栏 (1行)                                    │
/// └─────────────────────────────────────────────────┘
/// ```
pub(crate) struct Layout {
    pub(crate) title_bar: Rect,
    pub(crate) chat_view: Rect,
    pub(crate) tool_panel: Rect,
    pub(crate) input_bar: Rect,
    pub(crate) status_bar: Rect,
}

impl Layout {
    /// 按终端尺寸计算各区域位置。
    pub(crate) fn compute(terminal_width: u16, terminal_height: u16) -> Self {
        let title_h = 1;
        let input_h = 3;
        let status_h = 1;

        // 剩余空间分配给 chat_view + tool_panel
        let remaining = terminal_height
            .saturating_sub(title_h)
            .saturating_sub(input_h)
            .saturating_sub(status_h);

        // 工具面板占 35%，最少 20 列
        let tool_w = std::cmp::max(20, terminal_width * 35 / 100);
        let chat_w = terminal_width.saturating_sub(tool_w);

        Self {
            title_bar: Rect::new(0, 0, terminal_width, title_h),
            chat_view: Rect::new(0, title_h, chat_w, remaining),
            tool_panel: Rect::new(chat_w, title_h, tool_w, remaining),
            input_bar: Rect::new(0, title_h + remaining, terminal_width, input_h),
            status_bar: Rect::new(0, title_h + remaining + input_h, terminal_width, status_h),
        }
    }

    /// 不显示工具面板时的布局（全宽对话）。
    pub(crate) fn compute_no_panel(terminal_width: u16, terminal_height: u16) -> Self {
        let title_h = 1;
        let input_h = 3;
        let status_h = 1;

        let remaining = terminal_height
            .saturating_sub(title_h)
            .saturating_sub(input_h)
            .saturating_sub(status_h);

        Self {
            title_bar: Rect::new(0, 0, terminal_width, title_h),
            chat_view: Rect::new(0, title_h, terminal_width, remaining),
            tool_panel: Rect::ZERO,
            input_bar: Rect::new(0, title_h + remaining, terminal_width, input_h),
            status_bar: Rect::new(0, title_h + remaining + input_h, terminal_width, status_h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_inner_with_margin() {
        let r = Rect::new(1, 2, 10, 8);
        let inner = r.inner(1);
        assert_eq!(inner.x, 2);
        assert_eq!(inner.y, 3);
        assert_eq!(inner.width, 8);
        assert_eq!(inner.height, 6);
    }

    #[test]
    fn rect_zero_is_invalid() {
        assert!(!Rect::ZERO.is_valid());
    }

    #[test]
    fn layout_splits_correctly() {
        let layout = Layout::compute(80, 24);
        assert_eq!(layout.title_bar.height, 1);
        assert_eq!(layout.title_bar.y, 0);
        assert_eq!(layout.input_bar.height, 3);
        assert_eq!(layout.status_bar.height, 1);
        assert!(layout.chat_view.is_valid());
        assert!(layout.tool_panel.is_valid());
        // chat + tool 应等于 terminal_width
        assert_eq!(layout.chat_view.width + layout.tool_panel.width, 80);
    }

    #[test]
    fn layout_no_panel_gives_full_width_to_chat() {
        let layout = Layout::compute_no_panel(80, 24);
        assert_eq!(layout.chat_view.width, 80);
        assert!(!layout.tool_panel.is_valid());
    }

    #[test]
    fn layout_handles_tiny_terminal() {
        let layout = Layout::compute(20, 8);
        // 即使很小也应该有有效区域
        assert!(layout.title_bar.is_valid());
    }
}
