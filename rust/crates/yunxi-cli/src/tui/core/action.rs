#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Navigate(String),
    GoBack,
    GoForward,
    ShowDialog(String),
    HideDialog,
    Close,
    ToggleSidebar,
    SwitchTab(usize),
    NewSession,
    SwitchSession(String),
    DeleteSession(String),
    RenameSession(String, String),
    SaveSession,
    ExecuteCommand(String),
    ShowCommandPalette,
    HideCommandPalette,
    SwitchTheme(String),
    ToggleDarkMode,
    CopySelection,
    Paste,
    EditorCopy,
    EditorPaste,
    EditorCut,
    EditorUndo,
    EditorRedo,
    Quit,
    Refresh,
    ShowSubmenu(String, usize),
    ShowParentMenu(String),
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
