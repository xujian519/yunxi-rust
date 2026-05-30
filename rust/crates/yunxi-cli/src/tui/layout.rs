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

    /// 坐标是否落在本区域内（半开区间）。
    pub(crate) fn contains(self, col: u16, row: u16) -> bool {
        self.is_valid()
            && col >= self.x
            && col < self.x.saturating_add(self.width)
            && row >= self.y
            && row < self.y.saturating_add(self.height)
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
    /// 按终端尺寸计算各区域位置（默认 4 行输入区）。
    pub(crate) fn compute(terminal_width: u16, terminal_height: u16) -> Self {
        Self::compute_with_input_rows(terminal_width, terminal_height, 2, true)
    }

    /// 不显示工具面板时的布局（全宽对话）。
    pub(crate) fn compute_no_panel(terminal_width: u16, terminal_height: u16) -> Self {
        Self::compute_with_input_rows(terminal_width, terminal_height, 2, false)
    }

    /// 按内容行数动态计算输入区高度（含补全菜单占位）。
    pub(crate) fn compute_with_input_rows(
        terminal_width: u16,
        terminal_height: u16,
        input_content_rows: u16,
        with_tool_panel: bool,
    ) -> Self {
        let terminal_width = terminal_width.max(MIN_WIDTH);
        let terminal_height = terminal_height.max(MIN_HEIGHT);
        let title_h = 1u16;
        let input_h = input_block_height(input_content_rows);
        let status_h = 1u16;

        let remaining = terminal_height
            .saturating_sub(title_h)
            .saturating_sub(input_h)
            .saturating_sub(status_h);

        if with_tool_panel && terminal_width >= 40 {
            let tool_w = std::cmp::max(20, terminal_width * 35 / 100);
            let chat_w = terminal_width.saturating_sub(tool_w);
            Self {
                title_bar: Rect::new(0, 0, terminal_width, title_h),
                chat_view: Rect::new(0, title_h, chat_w, remaining),
                tool_panel: Rect::new(chat_w, title_h, tool_w, remaining),
                input_bar: Rect::new(0, title_h + remaining, terminal_width, input_h),
                status_bar: Rect::new(0, title_h + remaining + input_h, terminal_width, status_h),
            }
        } else {
            Self {
                title_bar: Rect::new(0, 0, terminal_width, title_h),
                chat_view: Rect::new(0, title_h, terminal_width, remaining),
                tool_panel: Rect::ZERO,
                input_bar: Rect::new(0, title_h + remaining, terminal_width, input_h),
                status_bar: Rect::new(0, title_h + remaining + input_h, terminal_width, status_h),
            }
        }
    }
}

const MIN_WIDTH: u16 = 20;
const MIN_HEIGHT: u16 = 8;

/// 输入区高度：内容行数 + 提示行 + 边框。
#[must_use]
pub(crate) fn input_block_height(content_rows: u16) -> u16 {
    content_rows.saturating_add(2).clamp(4, 12)
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
        assert_eq!(layout.input_bar.height, 4);
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
    fn rect_contains_point() {
        let r = Rect::new(10, 5, 20, 8);
        assert!(r.contains(10, 5));
        assert!(r.contains(29, 12));
        assert!(!r.contains(9, 5));
        assert!(!r.contains(30, 5));
    }

    #[test]
    fn layout_handles_tiny_terminal() {
        let layout = Layout::compute(20, 8);
        // 即使很小也应该有有效区域
        assert!(layout.title_bar.is_valid());
    }

    #[test]
    fn layout_clamps_to_minimum_size() {
        let layout = Layout::compute(5, 3);
        assert!(layout.chat_view.is_valid() || layout.chat_view.height == 0);
        assert!(layout.title_bar.is_valid());
    }

    #[test]
    fn layout_disables_tool_panel_on_narrow_width() {
        let layout = Layout::compute_with_input_rows(30, 20, 2, true);
        assert!(!layout.tool_panel.is_valid());
    }

    #[test]
    fn layout_survives_zero_size() {
        let layout = Layout::compute(0, 0);
        assert_eq!(layout.title_bar.width, MIN_WIDTH);
        assert!(layout.title_bar.is_valid());
    }
}
