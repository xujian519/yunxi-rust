// ── Core architecture (Architecture 2) ──
pub(crate) mod core;

// ─── Shared utility modules (kept from Architecture 1) ───

pub(crate) mod ansi;
pub(crate) mod clipboard;
pub(crate) mod color;
pub(crate) mod diff;
pub(crate) mod form;
pub(crate) mod frame;
pub(crate) mod markdown;
pub(crate) mod pager;
pub(crate) mod rich_text;
pub(crate) mod slash;
pub(crate) mod slash_complete;
pub(crate) mod spinner;
pub(crate) mod status_bar;
pub(crate) mod syntax;
pub(crate) mod tool_viz;

// ── Widget / Component modules ──
pub(crate) mod components;
pub(crate) mod widgets;

// ── Plugin / Theme / State (Architecture 2) ──
pub(crate) mod plugin;
pub(crate) mod plugins;
pub(crate) mod state;
pub(crate) mod theme;
pub(crate) mod ui_palette;

// ── Keymap ──
pub(crate) mod keymap;

// ── Layout ──
pub(crate) mod layout;

// ── Banner (used by live_cli) ──
pub(crate) mod banner;

// ── Turn / Runtime ──
pub(crate) mod turn;

// ── Entry point ──
pub(crate) mod runner;

// ── Router ──
pub(crate) mod router;

// ── Session / Workspace ──
pub(crate) mod session;
pub(crate) mod workspace;

// ── Progress ──
pub(crate) mod progress;

// ── Error ──
pub(crate) mod error;
