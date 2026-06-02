#![allow(dead_code)]

pub(crate) mod chat_view;
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
pub mod command_palette;
pub mod editor;
pub mod input;
pub mod label;
pub mod layout;
pub mod list;
pub mod menu;
pub mod progress_bar;
pub mod sidebar;
pub mod spacer;
pub mod spinner;
pub mod tab;
pub mod toast;
pub mod tree;

// TODO: Fix compilation errors in the following modules
// pub mod confirm;
// pub mod modal;
// pub mod picker;
// pub mod table;

pub use alert::{Alert, AlertAction, AlertLevel, AlertStyle};
pub use base::{generate_component_id, Component, ComponentState};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbStyle};
pub use button::{Button, ButtonStyle};
pub use command_palette::CommandPalette;
pub use label::Label;
pub use list::{List, ListItemData, ListStyle};
pub use menu::{Menu, MenuItem, MenuItemType, MenuStyle};
pub use progress_bar::{ProgressBar, ProgressBarStyle};
pub use sidebar::{Sidebar, SidebarItem, SidebarPosition, SidebarStyle};
pub use spacer::Spacer;
pub use spinner::{Spinner, SpinnerStyle};
pub use tab::{Tab, TabItem, TabStyle};
pub use toast::{Toast, ToastLevel, ToastMessage, ToastPosition, ToastStyle};
pub use tree::{Tree, TreeNode, TreeStyle};

#[cfg(test)]
pub mod tests;
