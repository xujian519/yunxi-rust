//! TUI 内 `/init`：项目初始化。

use std::env;
use std::path::Path;

use crate::format_report::init_yunxi_md;
use crate::tui::app::TuiApp;
use crate::tui::runner::TuiState;
use crate::tui::slash::{refresh_status, SlashDispatch};

fn is_patent_project_yunxi_md(cwd: &Path) -> bool {
    let path = cwd.join("YUNXI.md");
    std::fs::read_to_string(path)
        .ok()
        .is_some_and(|body| body.contains("patentCase:"))
}

/// 执行 `/init` 并在对话区给出摘要 + 分页详情。
pub(crate) fn dispatch_init_command(
    app: &mut TuiApp,
    state: &mut TuiState,
    width: u16,
    height: u16,
) -> Result<SlashDispatch, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;

    if is_patent_project_yunxi_md(&cwd) {
        app.push_system_message(
            "当前目录为专利案件项目（YUNXI.md 含 patentCase）。\n\
             专利材料扫描、多视图编辑与检索请使用桌面客户端（yunxi-desktop）。\n\
             终端仍可使用 /init 生成通用项目配置，或在本目录运行通用 yunxi 对话。",
        );
    }

    let body = init_yunxi_md()?;
    app.push_system_message("项目 /init 完成：已检查或生成 YUNXI.md、.yunxi 等配置。");
    app.push_output("Init", &body, width, height);

    refresh_status(app, state);
    Ok(SlashDispatch::Handled)
}
