#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // ── Navigation ──
    Navigate(String),
    GoBack,
    GoForward,

    // ── Dialog / Overlay ──
    ShowDialog(String),
    HideDialog,
    Close,

    // ── UI Toggle ──
    ToggleSidebar,
    ShowHelp,
    ShowGuide,
    ShowSessionPicker,
    OpenSessionPicker,
    SwitchTab(usize),

    // ── Session ──
    NewSession,
    SwitchSession(String),
    DeleteSession(String),
    RenameSession(String, String),
    SaveSession,

    // ── Commands ──
    ExecuteCommand(String),
    ShowCommandPalette,
    HideCommandPalette,

    // ── Theme ──
    SwitchTheme(String),
    ToggleDarkMode,

    // ── Editing / Clipboard ──
    CopySelection,
    Paste,
    EditorCopy,
    EditorPaste,
    EditorCut,
    EditorUndo,
    EditorRedo,

    // ── Turn / LLM ──
    Submit(String, bool),
    InterruptTurn,
    PermissionDecision(bool),
    FlowResume(String, String),

    // ── Lifecycle ──
    Quit,
    Refresh,

    // ── Menu ──
    ShowSubmenu(String, usize),
    ShowParentMenu(String),

    // ── Custom ──
    Custom(String),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Navigate(route) => write!(f, "Navigate({})", route),
            Action::ShowDialog(name) => write!(f, "ShowDialog({})", name),
            Action::ShowSubmenu(id, idx) => write!(f, "ShowSubmenu({}, {})", id, idx),
            Action::ShowParentMenu(id) => write!(f, "ShowParentMenu({})", id),
            Action::ExecuteCommand(cmd) => write!(f, "ExecuteCommand({})", cmd),
            Action::Submit(text, _) => write!(f, "Submit({}…)", &text[..text.len().min(30)]),
            Action::FlowResume(id, _) => write!(f, "FlowResume({})", id),
            Action::Custom(cmd) => write!(f, "Custom({})", cmd),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    Handled,
    Ignored,
    Action(Action),
    Actions(Vec<Action>),
}
