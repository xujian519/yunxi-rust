//! TUI slash command utilities — lightweight wrappers for new architecture.
//! The new App handles slash commands internally; this module provides
//! minimal helpers for external consumers if needed.

use crate::tui::core::app::App;
use crate::tui::status_bar::StatusBarSnapshot;

/// Refresh the status bar snapshot from the app state (thin wrapper).
pub(crate) fn refresh_status(app: &App) -> StatusBarSnapshot {
    StatusBarSnapshot {
        model: app.model().to_string(),
        permission_mode: app.permission_mode.as_str().to_string(),
        session_id: app.session_handle.id.clone(),
        ..StatusBarSnapshot::default()
    }
}

/// Shorthand for creating a new session picker.
pub(crate) fn create_session_picker(
    app: &App,
) -> Option<crate::tui::components::session_picker::SessionPicker> {
    use crate::session_mgr::list_managed_sessions;
    list_managed_sessions().ok().map(|sessions| {
        crate::tui::components::session_picker::SessionPicker::new(
            sessions,
            app.session_handle.id.clone(),
        )
    })
}
