//! 通用 TUI 模态覆盖层绘制。

use runtime::PermissionRequest;

use crate::session_meta::SuspendedFlowRecord;
use crate::tui::components::flow_hitl_overlay::FlowHitlOverlay;
use crate::tui::components::guide_overlay::GuideOverlay;
use crate::tui::components::help_overlay::HelpOverlay;
use crate::tui::components::permission_overlay::PermissionOverlay;
use crate::tui::frame::fit_lines;
use crate::tui::frame::Frame;
use crate::tui::layout::Rect;

pub(crate) struct ModalOverlays<'a> {
    pub width: u16,
    pub height: u16,
    pub show_help: bool,
    pub show_guide: bool,
    pub thinking: bool,
    pub pending_flow_hitl: Option<&'a SuspendedFlowRecord>,
    pub pending_permission: Option<&'a PermissionRequest>,
}

impl ModalOverlays<'_> {
    pub(crate) fn paint(&self, frame: &mut Frame) {
        let w = self.width;
        let h = self.height;

        if self.show_help {
            let overlay = Rect::new(w / 8, h / 6, w * 3 / 4, h * 2 / 3);
            let body = HelpOverlay::render(overlay);
            Self::overlay_lines(frame, overlay, &body);
        }

        if self.show_guide {
            let overlay = Rect::new(w / 6, h.saturating_sub(12).max(2), w * 2 / 3, 7);
            Self::overlay_lines(
                frame,
                overlay,
                &GuideOverlay::render(overlay, self.thinking),
            );
        }

        if let Some(record) = self.pending_flow_hitl {
            let overlay = Rect::new(w / 6, h / 3, w * 2 / 3, h / 3);
            Self::overlay_lines(frame, overlay, &FlowHitlOverlay::render(overlay, record));
        }

        if let Some(req) = self.pending_permission {
            let overlay = Rect::new(w / 6, h / 3, w * 2 / 3, h / 3);
            Self::overlay_lines(frame, overlay, &PermissionOverlay::render(overlay, req));
        }
    }

    fn overlay_lines(frame: &mut Frame, area: Rect, body: &str) {
        let lines = fit_lines(body, area.width as usize, area.height as usize);
        frame.overlay_lines(area, &lines);
    }
}
