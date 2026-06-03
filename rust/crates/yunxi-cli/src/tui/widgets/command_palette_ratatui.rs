use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::widgets::Clear;

use crate::tui::components::base::Component;
use crate::tui::components::command_palette::CommandPalette;

pub(crate) struct CommandPaletteWidget<'a> {
    pub palette: &'a CommandPalette,
}

impl CommandPaletteWidget<'_> {
    pub(crate) fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.palette.is_visible() {
            return;
        }

        Clear.render(area, buf);

        let popup_width = area.width.min(60).max(40);
        let popup_height = area.height.min(18).max(10);
        let popup = centered_rect(popup_width, popup_height, area);

        Component::render(self.palette, popup, buf);
    }
}

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(h)) / 2),
            Constraint::Length(h),
            Constraint::Length((area.height.saturating_sub(h)) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(w)) / 2),
            Constraint::Length(w),
            Constraint::Length((area.width.saturating_sub(w)) / 2),
        ])
        .split(v[1])[1]
}
