#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Navigate(String),
    GoBack,
    GoForward,
    ShowDialog(String),
    HideDialog,
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
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Navigate(route) => write!(f, "Navigate({})", route),
            Action::ShowDialog(name) => write!(f, "ShowDialog({})", name),
            Action::ExecuteCommand(cmd) => write!(f, "ExecuteCommand({})", cmd),
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
