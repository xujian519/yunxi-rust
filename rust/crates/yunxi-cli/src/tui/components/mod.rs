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
pub mod collapsible;
pub mod command_palette;
pub mod confirm;
pub mod editor;
pub mod input;
pub mod label;
pub mod layout;
pub mod list;
pub mod menu;
pub mod modal;
pub mod picker;
pub mod progress_bar;
pub mod progress_indicator;
pub mod sidebar;
pub mod spacer;
pub mod spinner;
pub mod tab;
pub mod table;
pub mod thinking_block;
pub mod toast;
pub mod tree;

// TODO: Fix compilation errors in the following modules
pub mod error_dialog;
pub mod form;
pub mod keymap_editor;
