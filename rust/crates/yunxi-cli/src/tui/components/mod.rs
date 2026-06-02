#![allow(dead_code)]

pub(crate) mod chat_view;
pub(crate) mod diff_view;
pub(crate) mod flow_hitl_overlay;
pub(crate) mod guide_overlay;
pub(crate) mod help_overlay;
pub(crate) mod input_bar;
pub(crate) mod permission_overlay;
pub(crate) mod session_picker;
pub(crate) mod tool_panel;

pub mod alert;
pub mod base;
pub mod breadcrumb;
pub mod button;
// pub mod collapsible;
pub mod command_palette;
// pub mod confirm;
pub mod editor;
pub mod input;
pub mod label;
pub mod layout;
pub mod list;
pub mod menu;
// pub mod modal;
// pub mod picker;
pub mod progress_bar;
pub mod progress_indicator;
pub mod sidebar;
pub mod spacer;
pub mod spinner;
pub mod tab;
// pub mod table;
// pub mod thinking_block;
pub mod toast;
pub mod tree;

// TODO: Fix compilation errors in the following modules
// pub mod error_dialog;
// pub mod form;
// pub mod keymap_editor;

pub use alert::{Alert, AlertAction, AlertLevel, AlertStyle};
pub use base::{generate_component_id, Component, ComponentState};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbStyle};
pub use button::{Button, ButtonStyle};
// pub use collapsible::{Collapsible, CollapsibleStyle};
pub use command_palette::CommandPalette;
// pub use confirm::Confirm;
// pub use error_dialog::{ErrorDialog, ErrorDialogAction};
// pub use form::{FieldType, Form, FormField};
// pub use keymap_editor::{KeyBinding, KeymapConflictResolution, KeymapEditor, RecordingState};
pub use label::Label;
pub use layout::Flex;
pub use list::{List, ListItemData, ListStyle, SelectionMode};
pub use menu::{Menu, MenuItem, MenuItemType, MenuStyle};
// pub use modal::Modal;
// pub use picker::Picker;
pub use progress_bar::{ProgressBar, ProgressBarStyle};
// pub use progress_indicator::{ProgressIndicator, ProgressStyle, ProgressType};
pub use sidebar::{Sidebar, SidebarItem, SidebarPosition, SidebarStyle};
pub use spacer::Spacer;
pub use spinner::{Spinner, SpinnerStyle};
pub use tab::{Tab, TabItem, TabStyle};
// pub use table::{Column, RowData, SortOrder, Table, TableStyle};
// pub use thinking_block::{ThinkingBlock, ThinkingBlockStyle, ThinkingStep};
pub use toast::{Toast, ToastLevel, ToastMessage, ToastPosition, ToastStyle};
pub use tree::{Tree, TreeNode, TreeStyle};

#[cfg(test)]
pub mod tests;
