#[derive(Debug, Clone)]
pub struct GlobalState {
    pub theme: ThemeState,
    pub session: SessionState,
    pub workspace: WorkspaceState,
    pub ui: UIState,
}

#[derive(Debug, Clone)]
pub struct ThemeState {
    pub current_theme: String,
    pub is_dark: bool,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub current_session_id: Option<String>,
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub current_workspace: Option<String>,
    pub workspaces: Vec<WorkspaceInfo>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct UIState {
    pub show_sidebar: bool,
    pub show_tool_panel: bool,
    pub sidebar_width: u16,
    pub tool_panel_width: u16,
}

pub trait Reducer<S, A> {
    fn reduce(&self, state: &mut S, action: A);
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            theme: ThemeState {
                current_theme: "default_dark".to_string(),
                is_dark: true,
            },
            session: SessionState {
                current_session_id: None,
                sessions: Vec::new(),
            },
            workspace: WorkspaceState {
                current_workspace: None,
                workspaces: Vec::new(),
            },
            ui: UIState {
                show_sidebar: true,
                show_tool_panel: true,
                sidebar_width: 30,
                tool_panel_width: 35,
            },
        }
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}
