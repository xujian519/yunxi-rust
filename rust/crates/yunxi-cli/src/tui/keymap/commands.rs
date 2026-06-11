use crate::tui::core::action::Action;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub action: CommandAction,
}

#[derive(Debug, Clone)]
pub enum CommandAction {
    Single(Action),
    Multiple(Vec<Action>),
    Dynamic(fn() -> Vec<Action>),
}

impl CommandAction {
    pub fn execute(&self) -> Vec<Action> {
        match self {
            CommandAction::Single(action) => vec![action.clone()],
            CommandAction::Multiple(actions) => actions.clone(),
            CommandAction::Dynamic(func) => func(),
        }
    }
}

impl From<Action> for CommandAction {
    fn from(action: Action) -> Self {
        CommandAction::Single(action)
    }
}

impl From<Vec<Action>> for CommandAction {
    fn from(actions: Vec<Action>) -> Self {
        CommandAction::Multiple(actions)
    }
}

impl Command {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        action: impl Into<CommandAction>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            action: action.into(),
        }
    }

    pub fn execute(&self) -> Vec<Action> {
        self.action.execute()
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };

        registry.register_builtin_commands();
        registry
    }

    fn register_builtin_commands(&mut self) {
        self.register(Command::new(
            "GoToTop",
            "跳转到顶部",
            Action::ShowDialog("go_to_top".to_string()),
        ));

        self.register(Command::new(
            "GoToBottom",
            "跳转到底部",
            Action::ShowDialog("go_to_bottom".to_string()),
        ));

        self.register(Command::new(
            "NavigateDown",
            "向下导航",
            Action::ShowDialog("navigate_down".to_string()),
        ));

        self.register(Command::new(
            "NavigateUp",
            "向上导航",
            Action::ShowDialog("navigate_up".to_string()),
        ));

        self.register(Command::new(
            "Help",
            "显示帮助信息",
            Action::ExecuteCommand("help".to_string()),
        ));

        self.register(Command::new("Quit", "退出应用", Action::Quit));

        self.register(Command::new(
            "ToggleSidebar",
            "切换侧边栏",
            Action::ToggleSidebar,
        ));

        self.register(Command::new(
            "ShowCommandPalette",
            "显示命令面板",
            Action::ShowCommandPalette,
        ));

        self.register(Command::new(
            "HideCommandPalette",
            "隐藏命令面板",
            Action::HideCommandPalette,
        ));

        self.register(Command::new("NewSession", "新建会话", Action::NewSession));

        self.register(Command::new("SaveSession", "保存会话", Action::SaveSession));

        self.register(Command::new(
            "ToggleDarkMode",
            "切换深色模式",
            Action::ToggleDarkMode,
        ));

        self.register(Command::new("Refresh", "刷新", Action::Refresh));

        self.register(Command::new("GoBack", "返回", Action::GoBack));

        self.register(Command::new("GoForward", "前进", Action::GoForward));

        self.register(Command::new("Copy", "复制", Action::CopySelection));

        self.register(Command::new("Paste", "粘贴", Action::Paste));

        self.register(Command::new("EditorUndo", "编辑器撤销", Action::EditorUndo));

        self.register(Command::new("EditorRedo", "编辑器重做", Action::EditorRedo));

        self.register(Command::new("StartSearch", "开始搜索", Action::StartSearch));

        self.register(Command::new("Collapse", "折叠节点", Action::Collapse));

        self.register(Command::new("Expand", "展开节点", Action::Expand));

        self.register(Command::new("SwitchModel", "切换模型", Action::SwitchModel));

        self.register(Command::new("ShowHelp", "显示帮助", Action::ShowHelp));
        self.register(Command::new("ShowGuide", "人机引导", Action::ShowGuide));
        self.register(Command::new("HideDialog", "关闭对话框", Action::HideDialog));
        self.register(Command::new(
            "InterruptTurn",
            "中断当前轮次",
            Action::InterruptTurn,
        ));

        self.register(Command::new(
            "ShowSessionPicker",
            "打开会话列表",
            Action::ShowSessionPicker,
        ));
        self.register(Command::new(
            "OpenSessionPicker",
            "打开会话列表",
            Action::OpenSessionPicker,
        ));

        self.register(Command::new(
            "ToggleToolPanel",
            "切换工具面板",
            Action::ShowDialog("toggle_tool_panel".to_string()),
        ));

        self.register(Command::new(
            "EditorCopy",
            "复制输入内容",
            Action::EditorCopy,
        ));
        self.register(Command::new("EditorCut", "剪切输入内容", Action::EditorCut));
        self.register(Command::new(
            "EditorPaste",
            "粘贴到输入",
            Action::EditorPaste,
        ));
    }

    pub fn register(&mut self, command: Command) {
        self.commands.insert(command.name.clone(), command);
    }

    pub fn get(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }

    pub fn execute(&self, name: &str) -> Option<Vec<Action>> {
        self.get(name).map(|cmd| cmd.execute())
    }

    pub fn list(&self) -> Vec<&Command> {
        self.commands.values().collect()
    }

    pub fn list_sorted(&self) -> Vec<&Command> {
        let mut list = self.list();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn search(&self, query: &str) -> Vec<&Command> {
        let query_lower = query.to_lowercase();
        self.list()
            .into_iter()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query_lower)
                    || cmd.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_command() {
        let mut registry = CommandRegistry::new();
        let cmd = Command::new(
            "Test",
            "Test command",
            Action::ExecuteCommand("test_action".to_string()),
        );
        registry.register(cmd.clone());
        assert_eq!(registry.get("Test").unwrap().name, "Test");
    }

    #[test]
    fn test_get_command() {
        let registry = CommandRegistry::new();
        assert!(registry.get("Help").is_some());
        assert!(registry.get("Unknown").is_none());
    }

    #[test]
    fn test_list_commands() {
        let registry = CommandRegistry::new();
        let list = registry.list();
        assert!(!list.is_empty());
    }

    #[test]
    fn test_list_sorted_commands() {
        let registry = CommandRegistry::new();
        let list = registry.list_sorted();
        assert!(!list.is_empty());

        for i in 1..list.len() {
            assert!(list[i - 1].name <= list[i].name);
        }
    }

    #[test]
    fn test_search_commands() {
        let registry = CommandRegistry::new();
        let results = registry.search("save");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|cmd| cmd.name.to_lowercase().contains("save")));
    }

    #[test]
    fn test_execute_command() {
        let registry = CommandRegistry::new();
        let actions = registry.execute("Quit");
        assert!(actions.is_some());
        assert_eq!(actions.unwrap().len(), 1);
    }

    #[test]
    fn test_command_action_single() {
        let action: CommandAction = Action::Quit.into();
        let actions = action.execute();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Quit);
    }

    #[test]
    fn test_command_action_multiple() {
        let actions: Vec<Action> = vec![Action::GoBack, Action::Refresh];
        let action: CommandAction = actions.clone().into();
        let result = action.execute();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_command_action_dynamic() {
        let action: CommandAction = CommandAction::Dynamic(|| vec![Action::Quit, Action::Refresh]);
        let result = action.execute();
        assert_eq!(result.len(), 2);
    }
}
