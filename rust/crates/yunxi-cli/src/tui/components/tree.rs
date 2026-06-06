use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::components::list::SelectionMode;
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub const ARROW_DOWN: &str = "▼";
pub const ARROW_RIGHT: &str = "▶";

pub struct Tree<T: Clone + ToString + Send + Sync> {
    state: ComponentState,
    nodes: Vec<TreeNode<T>>,
    expanded_paths: Vec<String>,
    focused_path: Option<String>,
    indent_size: usize,
    selected_paths: Vec<String>,
    selection_mode: SelectionMode,
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn Fn(String, &T) -> ActionResult + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_expand: Option<Box<dyn Fn(String, &T) -> ActionResult + Send + Sync>>,
    style: TreeStyle,
}

#[derive(Debug, Clone)]
pub struct TreeNode<T: Clone + ToString + Send + Sync> {
    pub path: String,
    pub value: T,
    pub label: String,
    pub is_leaf: bool,
    pub children: Vec<TreeNode<T>>,
}

#[derive(Debug, Clone)]
pub struct TreeStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub focused_bg: Color,
    pub focused_fg: Color,
    pub expanded_icon: String,
    pub collapsed_icon: String,
    pub leaf_icon: String,
    pub border: bool,
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            selected_bg: Color::Rgb(68, 138, 255),
            selected_fg: Color::Rgb(13, 13, 18),
            focused_bg: Color::Rgb(139, 176, 240),
            focused_fg: Color::Rgb(13, 13, 18),
            expanded_icon: ARROW_DOWN.to_string(),
            collapsed_icon: ARROW_RIGHT.to_string(),
            leaf_icon: "─".to_string(),
            border: true,
        }
    }
}

impl<T: Clone + ToString + Send + Sync> TreeNode<T> {
    pub fn new(path: impl Into<String>, value: T, label: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            value,
            label: label.into(),
            is_leaf: true,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<TreeNode<T>>) -> Self {
        self.is_leaf = children.is_empty();
        self.children = children;
        self
    }

    pub fn add_child(mut self, child: TreeNode<T>) -> Self {
        self.is_leaf = false;
        self.children.push(child);
        self
    }

    pub fn find_node(&self, path: &str) -> Option<&TreeNode<T>> {
        if self.path == path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(node) = child.find_node(path) {
                return Some(node);
            }
        }
        None
    }

    pub fn find_node_mut(&mut self, path: &str) -> Option<&mut TreeNode<T>> {
        if self.path == path {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(node) = child.find_node_mut(path) {
                return Some(node);
            }
        }
        None
    }
}

impl<T: Clone + ToString + Send + Sync> Tree<T> {
    pub fn new(nodes: Vec<TreeNode<T>>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("tree")),
            nodes,
            expanded_paths: Vec::new(),
            focused_path: None,
            indent_size: 2,
            selected_paths: Vec::new(),
            selection_mode: SelectionMode::Single,
            on_select: None,
            on_expand: None,
            style: TreeStyle::default(),
        }
    }

    pub fn with_nodes(mut self, nodes: Vec<TreeNode<T>>) -> Self {
        self.nodes = nodes;
        self
    }

    pub fn with_indent_size(mut self, indent_size: usize) -> Self {
        self.indent_size = indent_size;
        self
    }

    pub fn with_style(mut self, style: TreeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(String, &T) -> ActionResult + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn with_on_expand<F>(mut self, callback: F) -> Self
    where
        F: Fn(String, &T) -> ActionResult + Send + Sync + 'static,
    {
        self.on_expand = Some(Box::new(callback));
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn with_selection_mode(mut self, selection_mode: SelectionMode) -> Self {
        self.selection_mode = selection_mode;
        self
    }

    pub fn get_selection_mode(&self) -> SelectionMode {
        self.selection_mode
    }

    pub fn set_selection_mode(&mut self, mode: SelectionMode) {
        self.selection_mode = mode;
        if mode == SelectionMode::Single && self.selected_paths.len() > 1 {
            if let Some(path) = self.focused_path.clone() {
                self.selected_paths = vec![path];
            } else {
                self.selected_paths.clear();
            }
        }
    }

    pub fn is_expanded(&self, path: &str) -> bool {
        self.expanded_paths.contains(&path.to_string())
    }

    pub fn expand(&mut self, path: &str) {
        if !self.expanded_paths.contains(&path.to_string()) {
            self.expanded_paths.push(path.to_string());
        }
    }

    pub fn collapse(&mut self, path: &str) {
        self.expanded_paths.retain(|p| p != path);
    }

    pub fn toggle(&mut self, path: &str) {
        if self.is_expanded(path) {
            self.collapse(path);
        } else {
            self.expand(path);
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(ref path) = self.focused_path {
            match self.selection_mode {
                SelectionMode::Single => {
                    self.selected_paths.clear();
                    self.selected_paths.push(path.clone());
                }
                SelectionMode::Multiple => {
                    if self.selected_paths.contains(path) {
                        self.selected_paths.retain(|p| p != path);
                    } else {
                        self.selected_paths.push(path.clone());
                    }
                }
            }
        }
    }

    pub fn select_range(&mut self, start_path: &str, end_path: &str) {
        if self.selection_mode != SelectionMode::Multiple {
            return;
        }
        let visible_paths = self.get_all_visible_paths();
        let start_idx = visible_paths.iter().position(|p| p == start_path);
        let end_idx = visible_paths.iter().position(|p| p == end_path);

        if let (Some(start), Some(end)) = (start_idx, end_idx) {
            self.selected_paths.clear();
            let (start, end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            for i in start..=end {
                if let Some(path) = visible_paths.get(i) {
                    self.selected_paths.push(path.clone());
                }
            }
        }
    }

    pub fn select_all_visible(&mut self) {
        if self.selection_mode == SelectionMode::Multiple {
            self.selected_paths = self.get_all_visible_paths();
        }
    }

    pub fn expand_all(&mut self) {
        self.expanded_paths = self.collect_all_paths(&self.nodes, true);
    }

    pub fn collapse_all(&mut self) {
        self.expanded_paths.clear();
    }

    pub fn get_selected_paths(&self) -> &[String] {
        &self.selected_paths
    }

    pub fn get_selected_nodes(&self) -> Vec<&TreeNode<T>> {
        self.selected_paths
            .iter()
            .filter_map(|path| self.find_node(path))
            .collect()
    }

    pub fn get_focused_path(&self) -> Option<&String> {
        self.focused_path.as_ref()
    }

    pub fn find_node(&self, path: &str) -> Option<&TreeNode<T>> {
        for node in &self.nodes {
            if let Some(found) = node.find_node(path) {
                return Some(found);
            }
        }
        None
    }

    fn collect_all_paths(&self, nodes: &[TreeNode<T>], is_leaf: bool) -> Vec<String> {
        let mut paths = Vec::new();
        for node in nodes {
            if !is_leaf || node.is_leaf {
                paths.push(node.path.clone());
            }
            paths.extend(self.collect_all_paths(&node.children, is_leaf));
        }
        paths
    }

    fn flatten_nodes<'a>(
        &self,
        nodes: &'a [TreeNode<T>],
        depth: usize,
    ) -> Vec<(usize, String, &'a TreeNode<T>)> {
        let mut result = Vec::new();
        for node in nodes {
            result.push((depth, node.path.clone(), node));
            if !node.is_leaf && self.is_expanded(&node.path) {
                result.extend(self.flatten_nodes(&node.children, depth + 1));
            }
        }
        result
    }

    fn get_all_visible_paths(&self) -> Vec<String> {
        let flattened = self.flatten_nodes(&self.nodes, 0);
        flattened.iter().map(|(_, path, _)| path.clone()).collect()
    }

    fn navigate(&mut self, direction: i32) {
        let visible_paths = self.get_all_visible_paths();
        if visible_paths.is_empty() {
            return;
        }

        let current_index = self
            .focused_path
            .as_ref()
            .and_then(|p| visible_paths.iter().position(|x| x == p))
            .unwrap_or(0);

        let new_index = if direction > 0 {
            (current_index + 1).min(visible_paths.len() - 1)
        } else {
            current_index.saturating_sub(1)
        };

        self.focused_path = Some(visible_paths[new_index].clone());
    }

    fn navigate_with_range_selection(&mut self, direction: i32) {
        let old_path = self.focused_path.clone();
        self.navigate(direction);
        if let (Some(old), Some(new)) = (old_path, self.focused_path.clone()) {
            if self.selection_mode == SelectionMode::Multiple {
                self.select_range(&old, &new);
            }
        }
    }
}

impl<T: Clone + ToString + Send + Sync> Component for Tree<T> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let flattened = self.flatten_nodes(&self.nodes, 0);

        let lines: Vec<Line> = flattened
            .iter()
            .map(|(depth, path, node)| {
                let indent = " ".repeat(depth * self.indent_size);
                let icon = if node.is_leaf {
                    self.style.leaf_icon.as_str()
                } else if self.is_expanded(path) {
                    self.style.expanded_icon.as_str()
                } else {
                    self.style.collapsed_icon.as_str()
                };

                let style = if self.selected_paths.contains(path) {
                    Style::default()
                        .bg(self.style.selected_bg)
                        .fg(self.style.selected_fg)
                } else if self.focused_path.as_ref() == Some(path) && self.state.focused {
                    Style::default()
                        .bg(self.style.focused_bg)
                        .fg(self.style.focused_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(self.style.normal_bg)
                        .fg(self.style.normal_fg)
                };

                Line::from(vec![
                    Span::raw(indent),
                    Span::styled(icon.to_string() + " ", style),
                    Span::styled(node.label.clone(), style),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(if self.style.border {
                    Borders::ALL
                } else {
                    Borders::NONE
                })
                .style(Style::default()),
        );

        paragraph.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.navigate_with_range_selection(1);
                    } else {
                        self.navigate(1);
                    }
                    ActionResult::Handled
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.navigate_with_range_selection(-1);
                    } else {
                        self.navigate(-1);
                    }
                    ActionResult::Handled
                }
                KeyCode::Enter => {
                    if let Some(path) = self.focused_path.clone() {
                        let is_leaf = self.find_node(&path).map(|n| n.is_leaf).unwrap_or(true);
                        if !is_leaf {
                            self.toggle(&path);
                            ActionResult::Handled
                        } else {
                            ActionResult::Action(Action::Navigate(path))
                        }
                    } else {
                        ActionResult::Ignored
                    }
                }
                KeyCode::Char(' ') => {
                    self.toggle_selection();
                    ActionResult::Handled
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.select_all_visible();
                    ActionResult::Handled
                }
                KeyCode::Esc => {
                    self.selected_paths.clear();
                    ActionResult::Handled
                }
                KeyCode::Right => {
                    if let Some(path) = self.focused_path.clone() {
                        if let Some(node) = self.find_node(&path) {
                            if !node.is_leaf {
                                self.expand(&path);
                            }
                        }
                    }
                    ActionResult::Handled
                }
                KeyCode::Left => {
                    if let Some(path) = self.focused_path.clone() {
                        if self.is_expanded(&path) {
                            self.collapse(&path);
                        } else if let Some(parent_path) = path.rsplit('/').nth(1) {
                            self.focused_path = Some(format!("/{}", parent_path));
                        }
                    }
                    ActionResult::Handled
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.expand_all();
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
        if focused && self.focused_path.is_none() && !self.nodes.is_empty() {
            self.focused_path = Some(self.nodes[0].path.clone());
        }
    }

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let tree: Tree<String> = Tree::new(nodes);
        assert_eq!(tree.get_focused_path(), None);
    }

    #[test]
    fn test_tree_expansion() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child".to_string(), "child".to_string(), "Child"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes);
        tree.expand("/root");
        assert!(tree.is_expanded("/root"));
        tree.collapse("/root");
        assert!(!tree.is_expanded("/root"));
    }

    #[test]
    fn test_tree_navigation() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes);
        tree.on_focus(true);
        tree.toggle("/root");
        assert_eq!(tree.get_focused_path(), Some(&"/root".to_string()));

        tree.navigate(1);
        assert_eq!(tree.get_focused_path(), Some(&"/root/child1".to_string()));
    }

    #[test]
    fn test_tree_find_node() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child".to_string(), "child".to_string(), "Child"),
            ]),
        ];
        let tree: Tree<String> = Tree::new(nodes);
        assert!(tree.find_node("/root/child").is_some());
        assert!(tree.find_node("/nonexistent").is_none());
    }

    #[test]
    fn test_tree_selection() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes);
        tree.selected_paths.push("/root/child1".to_string());
        assert_eq!(tree.get_selected_paths().len(), 1);
    }

    #[test]
    fn test_tree_toggle_selection_single() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes).with_selection_mode(SelectionMode::Single);
        tree.on_focus(true);
        tree.toggle("/root");
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths(), vec!["/root"]);

        tree.navigate(1);
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths(), vec!["/root/child1"]);
    }

    #[test]
    fn test_tree_toggle_selection_multiple() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes).with_selection_mode(SelectionMode::Multiple);
        tree.on_focus(true);
        tree.toggle("/root");
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths(), vec!["/root"]);

        tree.navigate(1);
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths().len(), 2);

        tree.navigate(1);
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths().len(), 3);

        tree.navigate(-1);
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths().len(), 2);
    }

    #[test]
    fn test_tree_select_all_visible() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes).with_selection_mode(SelectionMode::Multiple);
        tree.on_focus(true);
        tree.toggle("/root");
        tree.select_all_visible();
        assert!(!tree.get_selected_paths().is_empty());
    }

    #[test]
    fn test_tree_clear_selection() {
        let nodes = vec![
            TreeNode::new("/root".to_string(), "root".to_string(), "Root").with_children(vec![
                TreeNode::new("/root/child1".to_string(), "child1".to_string(), "Child 1"),
                TreeNode::new("/root/child2".to_string(), "child2".to_string(), "Child 2"),
            ]),
        ];
        let mut tree: Tree<String> = Tree::new(nodes).with_selection_mode(SelectionMode::Multiple);
        tree.on_focus(true);
        tree.toggle("/root");
        tree.toggle_selection();
        assert_eq!(tree.get_selected_paths().len(), 1);

        tree.handle_event(&Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })));
        assert_eq!(tree.get_selected_paths().len(), 0);
    }
}
