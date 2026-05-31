//! REPL 模式下助手回复的流式 Markdown 输出。

use std::io::{self, Write};

use crossterm::cursor::MoveToColumn;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use runtime::TurnObserver;

use crate::render::{ColorTheme, MarkdownStreamState, Spinner, TerminalRenderer};

/// 将流式 Markdown 增量渲染并写入 stdout；可选绑定 spinner（等待首 token / 块间等待时旋转）。
pub(crate) struct ReplStreamObserver<'a> {
    renderer: &'a TerminalRenderer,
    stream: MarkdownStreamState,
    spinner: Option<ReplSpinner<'a>>,
}

struct ReplSpinner<'a> {
    spinner: &'a mut Spinner,
    theme: &'a ColorTheme,
    base_label: &'static str,
    /// 工具执行等阶段的动态提示。
    active_label: String,
    /// 是否已清除 spinner 行并开始输出正文。
    output_started: bool,
}

impl ReplSpinner<'_> {
    fn label(&self) -> &str {
        if self.active_label.is_empty() {
            self.base_label
        } else {
            &self.active_label
        }
    }
}

impl<'a> ReplStreamObserver<'a> {
    pub(crate) fn new(renderer: &'a TerminalRenderer) -> Self {
        Self {
            renderer,
            stream: MarkdownStreamState::default(),
            spinner: None,
        }
    }

    pub(crate) fn with_spinner(
        mut self,
        spinner: &'a mut Spinner,
        theme: &'a ColorTheme,
        label: &'static str,
    ) -> Self {
        self.spinner = Some(ReplSpinner {
            spinner,
            theme,
            base_label: label,
            active_label: String::new(),
            output_started: false,
        });
        self
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.clear_spinner_line()?;
        if let Some(chunk) = self.stream.flush(self.renderer) {
            self.mark_output_started();
            print!("{chunk}");
            if !chunk.ends_with('\n') {
                println!();
            }
            io::stdout().flush()?;
        }
        Ok(())
    }

    fn clear_spinner_line(&mut self) -> io::Result<()> {
        if let Some(slot) = &mut self.spinner {
            if !slot.output_started {
                execute!(io::stdout(), MoveToColumn(0), Clear(ClearType::CurrentLine))?;
                slot.output_started = true;
            }
        }
        Ok(())
    }

    fn mark_output_started(&mut self) {
        if let Some(slot) = &mut self.spinner {
            slot.output_started = true;
        }
    }

    fn tick_spinner(&mut self) -> io::Result<()> {
        if let Some(slot) = &mut self.spinner {
            if !slot.output_started {
                let label = slot.label().to_string();
                let _ = slot.spinner.tick(&label, slot.theme, &mut io::stdout());
            }
        }
        Ok(())
    }
}

impl TurnObserver for ReplStreamObserver<'_> {
    fn on_tool_use(&mut self, name: &str) {
        if let Some(slot) = &mut self.spinner {
            slot.active_label = format!("工具 {name} …");
            slot.output_started = false;
        }
        let _ = self.tick_spinner();
    }

    fn on_text_delta(&mut self, delta: &str) {
        if let Some(chunk) = self.stream.push(self.renderer, delta) {
            if let Some(slot) = &mut self.spinner {
                slot.active_label.clear();
            }
            let _ = self.clear_spinner_line();
            let _ = write!(io::stdout(), "{chunk}");
            let _ = io::stdout().flush();
        } else {
            let _ = self.tick_spinner();
        }
    }
}
